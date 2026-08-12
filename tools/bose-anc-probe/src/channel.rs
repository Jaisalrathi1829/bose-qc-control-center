//! RFCOMM channel with a controlled transmit path.
//!
//! Unlike `bose-rfcomm-listen`, this crate *can* transmit — that is its whole
//! purpose. The constraint here is different: `send_frame` is private to this
//! module's callers and every byte sequence passed to it comes from the
//! allowlist in `frames.rs`, which contains only sequences replayed verbatim
//! from an observed capture. There is no path from user input to arbitrary
//! bytes on the wire.

#![cfg(windows)]

use std::time::Duration;
use windows::core::GUID;
use windows::Win32::Devices::Bluetooth::{AF_BTH, BTHPROTO_RFCOMM, SOCKADDR_BTH};
use windows::Win32::Networking::WinSock::{
    closesocket, connect, recv, send, setsockopt, socket, WSACleanup, WSAGetLastError, WSAStartup,
    SEND_RECV_FLAGS, SOCKADDR, SOCKET, SOCKET_ERROR, SOCK_STREAM, WSADATA,
};

const SOL_SOCKET: i32 = 0xffff;
const SO_RCVTIMEO: i32 = 0x1006;
const WSAETIMEDOUT: i32 = 10060;

/// Option level for RFCOMM socket options — equal to `BTHPROTO_RFCOMM`.
const SOL_RFCOMM: i32 = 0x0003;
/// `SO_BTH_AUTHENTICATE` from ws2bth.h. Requires the link be authenticated.
const SO_BTH_AUTHENTICATE: i32 = 0x8000_0001u32 as i32;
/// `SO_BTH_ENCRYPT` from ws2bth.h. Requires the link be encrypted.
const SO_BTH_ENCRYPT: i32 = 0x0000_0002;

#[derive(Debug)]
pub enum ChannelError {
    Startup(i32),
    SocketCreate(i32),
    Connect(i32),
    Send(i32),
    Recv(i32),
}

impl std::fmt::Display for ChannelError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Startup(c) => write!(f, "WSAStartup failed ({c})"),
            Self::SocketCreate(c) => write!(f, "could not create an RFCOMM socket ({c})"),
            Self::Connect(c) => write!(f, "{}", explain_connect(*c)),
            Self::Send(c) => write!(f, "send failed ({c})"),
            Self::Recv(c) => write!(f, "recv failed ({c})"),
        }
    }
}

fn explain_connect(code: i32) -> String {
    let hint = match code {
        10060 => "the device did not answer. It may be asleep or out of range.",
        10061 => "the device refused the connection. The vendor channel is probably \
                  already held — close Bose Music on your phone and retry.",
        10064 => "the host is down. Power the headphones on.",
        10050 | 10051 => "the Bluetooth network is unreachable. Check Bluetooth is on.",
        _ => "see the Winsock error code.",
    };
    format!("RFCOMM connect failed ({code}): {hint}")
}

pub enum RecvOutcome {
    Data(Vec<u8>),
    Idle,
    Closed,
}

pub struct VendorChannel {
    socket: SOCKET,
}

/// Which endpoint to connect to.
pub enum Target {
    /// Resolve the RFCOMM channel from this service UUID via SDP.
    Service(GUID),
    /// Connect directly to an RFCOMM server channel number.
    ///
    /// Useful because a snoop log gives the DLCI, and server channel =
    /// DLCI >> 1. Connecting by number sidesteps any ambiguity about which
    /// endpoint an SDP lookup resolves to.
    Channel(u32),
}

impl VendorChannel {
    pub fn open_target(device_address: u64, target: Target, secure: bool) -> Result<Self, ChannelError> {
        match target {
            Target::Service(g) => Self::open_inner(device_address, g, 0, secure),
            Target::Channel(c) => Self::open_inner(device_address, GUID::zeroed(), c, secure),
        }
    }

    fn open_inner(device_address: u64, service: GUID, port: u32, secure: bool) -> Result<Self, ChannelError> {
        unsafe {
            let mut wsadata = WSADATA::default();
            let rc = WSAStartup(0x0202, &mut wsadata);
            if rc != 0 {
                return Err(ChannelError::Startup(rc));
            }

            let s = match socket(AF_BTH as i32, SOCK_STREAM, BTHPROTO_RFCOMM as i32) {
                Ok(s) => s,
                Err(_) => {
                    let e = WSAGetLastError().0;
                    WSACleanup();
                    return Err(ChannelError::SocketCreate(e));
                }
            };

            // Ask for an authenticated, encrypted link before connecting.
            //
            // The phone's session is authenticated; a bare Winsock connection
            // is not necessarily. Some devices accept an unencrypted RFCOMM
            // connection and then simply never reply on it, which matches
            // exactly what we observed. Failures here are not fatal — the
            // connection is still attempted, and the outcome is reported.
            if secure {
                let on: u32 = 1;
                let a = setsockopt(s, SOL_RFCOMM, SO_BTH_AUTHENTICATE, Some(&on.to_ne_bytes()));
                let e = setsockopt(s, SOL_RFCOMM, SO_BTH_ENCRYPT, Some(&on.to_ne_bytes()));
                if a == SOCKET_ERROR || e == SOCKET_ERROR {
                    eprintln!(
                        "  (note: could not request an authenticated/encrypted link; \
                         continuing without)"
                    );
                }
            }

            let addr = SOCKADDR_BTH {
                addressFamily: AF_BTH,
                btAddr: device_address,
                serviceClassId: service,
                port,
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
                return Err(ChannelError::Connect(e));
            }

            Ok(Self { socket: s })
        }
    }

    pub fn set_read_timeout(&self, timeout: Duration) {
        unsafe {
            let ms = timeout.as_millis().min(u32::MAX as u128) as u32;
            let _ = setsockopt(self.socket, SOL_SOCKET, SO_RCVTIMEO, Some(&ms.to_ne_bytes()));
        }
    }

    /// Transmits one allowlisted frame.
    ///
    /// Callers pass a constant from `frames.rs`. Winsock handles the RFCOMM
    /// framing, so these bytes become the RFCOMM payload directly — exactly
    /// what appeared as the payload in the capture.
    /// Returns how many bytes the stack accepted, so a partial or silently
    /// dropped write is visible rather than assumed successful.
    pub fn send_frame(&self, bytes: &[u8]) -> Result<usize, ChannelError> {
        unsafe {
            let n = send(self.socket, bytes, SEND_RECV_FLAGS(0));
            if n == SOCKET_ERROR {
                return Err(ChannelError::Send(WSAGetLastError().0));
            }
            Ok(n as usize)
        }
    }

    pub fn receive(&self) -> Result<RecvOutcome, ChannelError> {
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
                Err(ChannelError::Recv(e))
            }
        }
    }
}

impl Drop for VendorChannel {
    fn drop(&mut self) {
        unsafe {
            let _ = closesocket(self.socket);
            WSACleanup();
        }
    }
}

pub fn parse_address(hex: &str) -> Option<u64> {
    let cleaned: String = hex.chars().filter(|c| c.is_ascii_hexdigit()).collect();
    if cleaned.len() != 12 {
        return None;
    }
    u64::from_str_radix(&cleaned, 16).ok()
}
