//! btsnoop file format.
//!
//! Android writes Bluetooth HCI traces in the btsnoop format, which is Sun's
//! snoop format (RFC 1761) with a Bluetooth datalink type. Layout:
//!
//! ```text
//! Header  (16 bytes)
//!   "btsnoop\0"          8 bytes
//!   version              4 bytes, big-endian, always 1
//!   datalink type        4 bytes, big-endian, 1002 = H4 UART
//!
//! Record (25+ bytes), repeated
//!   original length      4 bytes, big-endian
//!   included length      4 bytes, big-endian
//!   packet flags         4 bytes, big-endian
//!   cumulative drops     4 bytes, big-endian
//!   timestamp            8 bytes, big-endian, microseconds since 0000-01-01
//!   packet data          `included length` bytes
//! ```
//!
//! Every integer is big-endian, which is easy to get wrong because the HCI
//! payload inside each record is little-endian.

pub const MAGIC: &[u8; 8] = b"btsnoop\0";

/// Microseconds between 0000-01-01 and the Unix epoch. btsnoop timestamps are
/// measured from the former; converting lets us print wall-clock times.
const EPOCH_OFFSET_US: i64 = 0x00dc_ddb3_0f2f_8000;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Direction {
    /// Host to controller — the phone sending to the headphones.
    Sent,
    /// Controller to host — the headphones replying.
    Received,
}

impl Direction {
    pub fn label(self) -> &'static str {
        match self {
            Self::Sent => "PHONE->DEVICE",
            Self::Received => "DEVICE->PHONE",
        }
    }
}

#[derive(Debug, Clone)]
pub struct Record {
    /// Microseconds since the Unix epoch. Negative values are possible for
    /// malformed logs and are passed through rather than clamped.
    pub timestamp_us: i64,
    pub direction: Direction,
    /// H4 packet, starting with the packet-type byte.
    pub data: Vec<u8>,
}

#[derive(Debug)]
pub enum ParseError {
    TooShort,
    BadMagic,
    UnsupportedDatalink(u32),
}

impl std::fmt::Display for ParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::TooShort => write!(f, "file is too short to be a btsnoop log"),
            Self::BadMagic => write!(
                f,
                "not a btsnoop log (missing the \"btsnoop\\0\" signature). \
                 If this came from a bug report, extract \
                 FS/data/misc/bluetooth/logs/btsnoop_hci.log from the zip first."
            ),
            Self::UnsupportedDatalink(d) => write!(
                f,
                "unsupported datalink type {d}; only 1002 (H4 UART) is handled"
            ),
        }
    }
}

fn be_u32(b: &[u8]) -> u32 {
    u32::from_be_bytes([b[0], b[1], b[2], b[3]])
}

fn be_i64(b: &[u8]) -> i64 {
    i64::from_be_bytes([b[0], b[1], b[2], b[3], b[4], b[5], b[6], b[7]])
}

/// Parses an entire btsnoop file into records.
///
/// Truncated trailing records are ignored rather than treated as an error —
/// snoop logs are frequently cut off mid-record when the capture is stopped.
pub fn parse(bytes: &[u8]) -> Result<Vec<Record>, ParseError> {
    if bytes.len() < 16 {
        return Err(ParseError::TooShort);
    }
    if &bytes[..8] != MAGIC {
        return Err(ParseError::BadMagic);
    }
    let datalink = be_u32(&bytes[12..16]);
    if datalink != 1002 {
        return Err(ParseError::UnsupportedDatalink(datalink));
    }

    let mut records = Vec::new();
    let mut off = 16;

    while off + 24 <= bytes.len() {
        let included = be_u32(&bytes[off + 4..off + 8]) as usize;
        let flags = be_u32(&bytes[off + 8..off + 12]);
        let ts = be_i64(&bytes[off + 16..off + 24]);
        let start = off + 24;
        let end = start + included;
        if end > bytes.len() {
            break; // truncated tail
        }

        records.push(Record {
            timestamp_us: ts - EPOCH_OFFSET_US,
            // Bit 0: 0 = host to controller (sent), 1 = controller to host.
            direction: if flags & 0x01 == 0 {
                Direction::Sent
            } else {
                Direction::Received
            },
            data: bytes[start..end].to_vec(),
        });
        off = end;
    }

    Ok(records)
}

/// Formats a Unix-epoch microsecond timestamp as `HH:MM:SS.mmm` UTC.
pub fn format_time(us: i64) -> String {
    if us < 0 {
        return "??:??:??.???".to_string();
    }
    let total_secs = us / 1_000_000;
    let millis = (us % 1_000_000) / 1000;
    let secs = total_secs % 60;
    let mins = (total_secs / 60) % 60;
    let hours = (total_secs / 3600) % 24;
    format!("{hours:02}:{mins:02}:{secs:02}.{millis:03}")
}

#[cfg(test)]
mod tests {
    use super::*;

    fn build(records: &[(u32, i64, &[u8])]) -> Vec<u8> {
        let mut out = Vec::new();
        out.extend_from_slice(MAGIC);
        out.extend_from_slice(&1u32.to_be_bytes());
        out.extend_from_slice(&1002u32.to_be_bytes());
        for (flags, ts, data) in records {
            out.extend_from_slice(&(data.len() as u32).to_be_bytes());
            out.extend_from_slice(&(data.len() as u32).to_be_bytes());
            out.extend_from_slice(&flags.to_be_bytes());
            out.extend_from_slice(&0u32.to_be_bytes());
            out.extend_from_slice(&ts.to_be_bytes());
            out.extend_from_slice(data);
        }
        out
    }

    #[test]
    fn rejects_a_non_btsnoop_file() {
        assert!(matches!(parse(b"not a log at all!!"), Err(ParseError::BadMagic)));
    }

    #[test]
    fn rejects_short_input() {
        assert!(matches!(parse(b"btsno"), Err(ParseError::TooShort)));
    }

    #[test]
    fn rejects_unsupported_datalink() {
        let mut f = Vec::new();
        f.extend_from_slice(MAGIC);
        f.extend_from_slice(&1u32.to_be_bytes());
        f.extend_from_slice(&99u32.to_be_bytes());
        assert!(matches!(parse(&f), Err(ParseError::UnsupportedDatalink(99))));
    }

    #[test]
    fn reads_direction_from_the_flag_bit() {
        let f = build(&[(0, EPOCH_OFFSET_US, &[0x02, 0xAA]), (1, EPOCH_OFFSET_US, &[0x02, 0xBB])]);
        let recs = parse(&f).unwrap();
        assert_eq!(recs.len(), 2);
        assert_eq!(recs[0].direction, Direction::Sent);
        assert_eq!(recs[1].direction, Direction::Received);
    }

    #[test]
    fn converts_timestamps_to_the_unix_epoch() {
        let f = build(&[(0, EPOCH_OFFSET_US, &[0x02])]);
        assert_eq!(parse(&f).unwrap()[0].timestamp_us, 0);
    }

    /// Captures stopped mid-write leave a partial final record. Losing it is
    /// fine; failing the whole parse because of it is not.
    #[test]
    fn ignores_a_truncated_trailing_record() {
        let mut f = build(&[(0, EPOCH_OFFSET_US, &[0x02, 0x01, 0x02])]);
        f.extend_from_slice(&[0, 0, 0, 99, 0, 0, 0, 99]); // header claiming 99 bytes
        let recs = parse(&f).unwrap();
        assert_eq!(recs.len(), 1);
    }

    #[test]
    fn formats_times() {
        assert_eq!(format_time(0), "00:00:00.000");
        assert_eq!(format_time(3_661_500_000), "01:01:01.500");
    }
}
