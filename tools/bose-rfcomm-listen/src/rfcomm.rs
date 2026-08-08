//! Listen-only RFCOMM socket.
//!
//! # The no-write guarantee
//!
//! This module never transmits. That is enforced structurally rather than by
//! discipline: `send`, `WSASend`, `sendto` and `WSASendTo` are **not imported**
//! anywhere in this crate, so there is no symbol in scope that could transmit
//! application data. The only socket operations used are:
//!
//!   `WSAStartup` `socket` `setsockopt` `connect` `recv` `shutdown` `closesocket`
//!
//! `connect` performs the RFCOMM channel establishment handshake, which is
//! link-layer protocol, not application data. Zero application bytes are sent.
//!
//! # A note on `shutdown(SD_SEND)`
//!
//! Half-closing the transmit direction is available via [`OpenOptions`], and
//! adds OS-level enforcement on top of the structural guarantee above. It is
//! **off by default**, because it was measured to break the capture: the test
//! headphones closed the channel 16ms after connect when it was applied.
//!
//! That is the expected consequence, in hindsight. `shutdown(SD_SEND)` signals
//! to the peer that we will never transmit. A device speaking a
//! request/response protocol has no reason to hold a channel open for a peer
//! that has announced it will never ask anything, so it hangs up. The
//! half-close was measuring our own signal, not the device's behaviour.
//!
//! With it off, the no-write guarantee still holds — transmitting requires a
//! `send` call, and no such symbol exists in this crate.

#![cfg(windows)]

use std::time::Duration;
use windows::core::GUID;
use windows::Win32::Devices::Bluetooth::{AF_BTH, BTHPROTO_RFCOMM, SOCKADDR_BTH};
use windows::Win32::Networking::WinSock::{
    closesocket, connect, recv, setsockopt, shutdown, socket, WSACleanup, WSAGetLastError,
    WSAStartup, SD_SEND, SEND_RECV_FLAGS, SOCKADDR, SOCKET, SOCKET_ERROR, SOCK_STREAM, WSADATA,
};

/// `SOL_SOCKET`, from winsock2.h. Used numerically to keep the imported
/// surface of this module as small as possible.
const SOL_SOCKET: i32 = 0xffff;
/// `SO_RCVTIMEO`, from winsock2.h.
const SO_RCVTIMEO: i32 = 0x1006;
/// `WSAETIMEDOUT`, returned by `recv` when the receive timeout elapses.
const WSAETIMEDOUT: i32 = 10060;

#[derive(Debug)]
pub enum RfcommError {
    Startup(i32),
    SocketCreate(i32),
    Connect(i32),
    Recv(i32),
}

impl std::fmt::Display for RfcommError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Startup(c) => write!(f, "WSAStartup failed ({c})"),
            Self::SocketCreate(c) => write!(f, "could not create an RFCOMM socket ({c})"),
            Self::Connect(c) => write!(f, "{}", explain_connect_error(*c)),
            Self::Recv(c) => write!(f, "recv failed ({c})"),
        }
    }
}

impl std::error::Error for RfcommError {}

/// Turns a Winsock connect error into something actionable.
fn explain_connect_error(code: i32) -> String {
    let hint = match code {
        10060 => "the device did not answer (timed out). It may be asleep or out of range.",
        10061 => "the device refused the connection. The service may already be in use \
                  — check whether Bose Music is connected, and close it.",
        10064 => "the host is down. Power the headphones on.",
        10050 | 10051 => "the Bluetooth network is unreachable. Check that Bluetooth is on.",
        10013 => "permission denied.",
        10022 => "invalid argument — the service UUID may not be advertised by this device.",
        _ => "see the Winsock error code for details.",
    };
    format!("RFCOMM connect failed ({code}): {hint}")
}

/// What a single `recv` produced.
pub enum RecvOutcome {
    /// The device voluntarily sent us bytes.
    Data(Vec<u8>),
    /// Nothing arrived within the timeout. Normal and expected.
    Idle,
    /// The device closed the channel.
    Closed,
}

/// How to open the channel.
#[derive(Debug, Clone, Copy, Default)]
pub struct OpenOptions {
    /// Call `shutdown(SD_SEND)` after connecting.
    ///
    /// Adds OS-level enforcement of the no-write guarantee, at the cost of
    /// telling the device we will never speak — which makes a request/response
    /// peer hang up immediately. Off by default; see the module docs.
    pub half_close_transmit: bool,
}

/// A connected, receive-only RFCOMM channel.
pub struct ListenOnlyChannel {
    socket: SOCKET,
}

impl ListenOnlyChannel {
    /// Opens the given service on the given device address.
    ///
    /// `service` is resolved through SDP by Windows, so the RFCOMM channel
    /// number does not need to be known in advance.
    pub fn open(
        device_address: u64,
        service: GUID,
        options: OpenOptions,
    ) -> Result<Self, RfcommError> {
        unsafe {
            let mut wsadata = WSADATA::default();
            let rc = WSAStartup(0x0202, &mut wsadata);
            if rc != 0 {
                return Err(RfcommError::Startup(rc));
            }

            let s = match socket(AF_BTH as i32, SOCK_STREAM, BTHPROTO_RFCOMM as i32) {
                Ok(s) => s,
                Err(_) => {
                    let e = WSAGetLastError().0;
                    WSACleanup();
                    return Err(RfcommError::SocketCreate(e));
                }
            };

            // port = 0 with a serviceClassId set tells Windows to look the
            // channel up over SDP rather than us guessing a channel number.
            let addr = SOCKADDR_BTH {
                addressFamily: AF_BTH,
                btAddr: device_address,
                serviceClassId: service,
                port: 0,
            };

            let rc = connect(
                s,
                &addr as *const SOCKADDR_BTH as *const SOCKADDR,
                std::mem::size_of::<SOCKADDR_BTH>() as i32,
            );
            if rc == SOCKET_ERROR {
                let e = WSAGetLastError().0;
                let _ = closesocket(s);
                WSACleanup();
                return Err(RfcommError::Connect(e));
            }

            if options.half_close_transmit {
                // Opt-in only. Measured to cause the test headphones to close
                // the channel after 16ms, because it announces that we will
                // never transmit.
                let _ = shutdown(s, SD_SEND);
            }

            Ok(Self { socket: s })
        }
    }

    /// Sets how long `receive` blocks before reporting `Idle`.
    pub fn set_read_timeout(&self, timeout: Duration) {
        unsafe {
            let ms = timeout.as_millis().min(u32::MAX as u128) as u32;
            let bytes = ms.to_ne_bytes();
            let _ = setsockopt(self.socket, SOL_SOCKET, SO_RCVTIMEO, Some(&bytes));
        }
    }

    /// Reads whatever the device has voluntarily sent. Never transmits.
    pub fn receive(&self) -> Result<RecvOutcome, RfcommError> {
        let mut buf = [0u8; 4096];
        unsafe {
            let n = recv(self.socket, &mut buf, SEND_RECV_FLAGS(0));
            if n > 0 {
                return Ok(RecvOutcome::Data(buf[..n as usize].to_vec()));
            }
            if n == 0 {
                return Ok(RecvOutcome::Closed);
            }
            let e = WSAGetLastError().0;
            if e == WSAETIMEDOUT {
                Ok(RecvOutcome::Idle)
            } else {
                Err(RfcommError::Recv(e))
            }
        }
    }
}

impl Drop for ListenOnlyChannel {
    fn drop(&mut self) {
        unsafe {
            let _ = closesocket(self.socket);
            WSACleanup();
        }
    }
}

/// Parses a 12-hex-digit Bluetooth address into the `BTH_ADDR` form Winsock
/// expects (a plain 48-bit integer in the low bits of a u64).
pub fn parse_address(hex: &str) -> Option<u64> {
    let cleaned: String = hex.chars().filter(|c| c.is_ascii_hexdigit()).collect();
    if cleaned.len() != 12 {
        return None;
    }
    u64::from_str_radix(&cleaned, 16).ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_a_plain_address() {
        assert_eq!(parse_address("E458BCF9F02E"), Some(0xE458BCF9F02E));
    }

    #[test]
    fn tolerates_separators() {
        assert_eq!(parse_address("E4:58:BC:F9:F0:2E"), Some(0xE458BCF9F02E));
    }

    #[test]
    fn rejects_wrong_length() {
        assert_eq!(parse_address("E458BCF9F0"), None);
        assert_eq!(parse_address(""), None);
    }
}
