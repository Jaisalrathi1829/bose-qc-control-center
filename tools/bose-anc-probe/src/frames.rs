//! The allowlist of frames this tool may transmit.
//!
//! Every byte sequence here was **observed being sent by Bose Music** in the
//! Experiment 3 snoop capture and is replayed verbatim. Nothing is
//! constructed from a theory about the protocol's length semantics, because
//! that theory is not fully settled — the observed `1F 03 05 02 00` carries a
//! length byte of `0x02` with a single payload byte, which does not match the
//! layout the rest of the group follows.
//!
//! Replaying exact observed bytes sidesteps that uncertainty entirely. There
//! is no function here that builds a frame from arbitrary input.

/// Noise-control modes, with the indices the device reported for them.
///
/// The names come from the device itself: `1F 06` returned each mode as a
/// fixed-size block containing its name in ASCII.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NoiseMode {
    Quiet,
    Aware,
    Home,
}

impl NoiseMode {
    pub fn index(self) -> u8 {
        match self {
            Self::Quiet => 0x00,
            Self::Aware => 0x01,
            Self::Home => 0x02,
        }
    }

    pub fn name(self) -> &'static str {
        match self {
            Self::Quiet => "Quiet",
            Self::Aware => "Aware",
            Self::Home => "Home",
        }
    }

    pub fn from_index(i: u8) -> Option<Self> {
        Some(match i {
            0x00 => Self::Quiet,
            0x01 => Self::Aware,
            0x02 => Self::Home,
            _ => return None,
        })
    }

    pub fn parse(s: &str) -> Option<Self> {
        Some(match s.trim().to_lowercase().as_str() {
            "quiet" => Self::Quiet,
            "aware" => Self::Aware,
            "home" => Self::Home,
            _ => return None,
        })
    }
}

/// Protocol version request — the first frame Bose Music sends.
///
/// Observed at 14:27:55.635 as `00 01 01`, answered with
/// `00 01 03 05 31 2E 31 2E 30` — length 5, payload `"1.1.0"` in ASCII.
///
/// A read of the noise mode sent without this produced no reply at all, which
/// suggests the device ignores requests until the session has been opened.
pub const PROTOCOL_VERSION_REQUEST: [u8; 3] = [0x00, 0x01, 0x01];

/// Enumerate the noise-control function group.
///
/// Observed at 14:27:55.859 as `1F 01 05`, answered with the 246-byte block
/// that lists the available modes. Bose Music always issues this before
/// touching `1F 03`, so it may be required to activate the group.
pub const NOISE_GROUP_ENUMERATE: [u8; 3] = [0x1F, 0x01, 0x05];

/// Read the current noise-control mode.
///
/// Observed at 14:28:06.819 as `PHONE->DEVICE 1F 03 01 00`, answered with
/// `1F 03 03 01 01`. This changes nothing on the device.
pub const READ_CURRENT_MODE: [u8; 4] = [0x1F, 0x03, 0x01, 0x00];

/// Set the current noise-control mode.
///
/// Observed six times in the capture as `1F 03 05 02 XX`, where the final
/// byte selects the mode. Only the three modes the device actually named are
/// permitted; there is no path to send an arbitrary index.
pub fn set_mode(mode: NoiseMode) -> [u8; 5] {
    [0x1F, 0x03, 0x05, 0x02, mode.index()]
}

/// Extracts a mode index from a device response.
///
/// Accepts the two shapes observed carrying the current mode:
///   `1F 03 03 01 XX`  — status, in reply to a read
///   `1F 03 06 01 XX`  — notify, sent after a change
///
/// Frames are frequently concatenated into one RFCOMM payload, so this scans
/// rather than assuming the interesting frame is first.
pub fn find_reported_mode(payload: &[u8]) -> Option<u8> {
    let mut i = 0;
    while i + 4 < payload.len() {
        if payload[i] == 0x1F
            && payload[i + 1] == 0x03
            && (payload[i + 2] == 0x03 || payload[i + 2] == 0x06)
            && payload[i + 3] == 0x01
        {
            return Some(payload[i + 4]);
        }
        i += 1;
    }
    None
}

pub fn to_hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02X}")).collect::<Vec<_>>().join(" ")
}

#[cfg(test)]
mod tests {
    use super::*;

    /// These must match the capture byte for byte.
    #[test]
    fn frames_match_the_observed_bytes() {
        assert_eq!(to_hex(&READ_CURRENT_MODE), "1F 03 01 00");
        assert_eq!(to_hex(&set_mode(NoiseMode::Quiet)), "1F 03 05 02 00");
        assert_eq!(to_hex(&set_mode(NoiseMode::Aware)), "1F 03 05 02 01");
        assert_eq!(to_hex(&set_mode(NoiseMode::Home)), "1F 03 05 02 02");
    }

    #[test]
    fn parses_a_status_response() {
        // Observed: 1F 03 03 01 01 -> Aware
        assert_eq!(find_reported_mode(&[0x1F, 0x03, 0x03, 0x01, 0x01]), Some(0x01));
    }

    #[test]
    fn parses_a_notify_response() {
        assert_eq!(find_reported_mode(&[0x1F, 0x03, 0x06, 0x01, 0x00]), Some(0x00));
    }

    /// Responses often arrive concatenated; the mode frame may not be first.
    #[test]
    fn finds_the_mode_frame_inside_a_concatenated_payload() {
        let payload = [
            0x1F, 0x03, 0x07, 0x00, // result
            0x1F, 0x04, 0x03, 0x01, 0x00, // some other notify
            0x1F, 0x03, 0x06, 0x01, 0x00, // current mode = Quiet
        ];
        assert_eq!(find_reported_mode(&payload), Some(0x00));
    }

    #[test]
    fn ignores_unrelated_payloads() {
        assert_eq!(find_reported_mode(&[0x05, 0x06, 0x03, 0x01, 0x01]), None);
        assert_eq!(find_reported_mode(&[]), None);
        assert_eq!(find_reported_mode(&[0x1F, 0x03]), None);
    }

    #[test]
    fn mode_round_trips() {
        for m in [NoiseMode::Quiet, NoiseMode::Aware, NoiseMode::Home] {
            assert_eq!(NoiseMode::from_index(m.index()), Some(m));
        }
        assert_eq!(NoiseMode::from_index(0x03), None);
    }

    #[test]
    fn parses_mode_names() {
        assert_eq!(NoiseMode::parse("quiet"), Some(NoiseMode::Quiet));
        assert_eq!(NoiseMode::parse("AWARE"), Some(NoiseMode::Aware));
        assert_eq!(NoiseMode::parse("nonsense"), None);
    }
}
