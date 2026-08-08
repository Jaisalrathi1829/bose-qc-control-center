//! Capture log, export, and correlation.
//!
//! A deliberate constraint runs through this module: it describes what was
//! observed and never says what it means. Frames are grouped and counted, and
//! markers are correlated by time, but nothing here labels a byte pattern as a
//! "command", a "capability", or an ANC state. Temporal correlation is not
//! causation, and a frame that changes when you press a button is evidence
//! that something changed — not evidence of what.

use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

/// Physical actions the operator can mark.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum Action {
    AncQuiet,
    AncAware,
    AncCustom,
    VolumeUp,
    VolumeDown,
    Play,
    Pause,
    ButtonPress,
    Power,
    Connect,
    Disconnect,
    Other,
}

impl Action {
    /// Single-key codes the operator types during a capture.
    pub fn from_key(k: &str) -> Option<Self> {
        Some(match k.trim().to_lowercase().as_str() {
            "1" => Self::AncQuiet,
            "2" => Self::AncAware,
            "3" => Self::AncCustom,
            "u" => Self::VolumeUp,
            "d" => Self::VolumeDown,
            "p" => Self::Play,
            "s" => Self::Pause,
            "b" => Self::ButtonPress,
            "w" => Self::Power,
            "c" => Self::Connect,
            "x" => Self::Disconnect,
            "o" => Self::Other,
            _ => return None,
        })
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::AncQuiet => "ANC_QUIET",
            Self::AncAware => "ANC_AWARE",
            Self::AncCustom => "ANC_CUSTOM",
            Self::VolumeUp => "VOLUME_UP",
            Self::VolumeDown => "VOLUME_DOWN",
            Self::Play => "PLAY",
            Self::Pause => "PAUSE",
            Self::ButtonPress => "BUTTON_PRESS",
            Self::Power => "POWER",
            Self::Connect => "CONNECT",
            Self::Disconnect => "DISCONNECT",
            Self::Other => "OTHER",
        }
    }
}

pub const KEY_HELP: &str = "\
  1 ANC Quiet      2 ANC Aware     3 ANC Custom
  u Volume Up      d Volume Down
  p Play           s Pause
  b Button press   w Power
  c Connect        x Disconnect    o Other
  q Quit and export";

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "kind", rename_all = "camelCase")]
pub enum Event {
    /// Bytes the device sent us. Direction is always device-to-host: this tool
    /// cannot transmit.
    #[serde(rename_all = "camelCase")]
    Frame {
        /// Milliseconds since capture start.
        at_ms: u128,
        timestamp: String,
        direction: &'static str,
        service_uuid: String,
        length: usize,
        hex: String,
        ascii: String,
    },
    /// A physical action the operator performed and marked.
    #[serde(rename_all = "camelCase")]
    Marker {
        at_ms: u128,
        timestamp: String,
        action: Action,
    },
    #[serde(rename_all = "camelCase")]
    Note {
        at_ms: u128,
        timestamp: String,
        text: String,
    },
}

impl Event {
    /// Kept for callers analysing an imported capture file.
    #[allow(dead_code)]
    pub fn at_ms(&self) -> u128 {
        match self {
            Self::Frame { at_ms, .. } | Self::Marker { at_ms, .. } | Self::Note { at_ms, .. } => {
                *at_ms
            }
        }
    }
}

pub fn to_hex(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|b| format!("{b:02X}"))
        .collect::<Vec<_>>()
        .join(" ")
}

/// Printable rendering, with non-printable bytes shown as `.`.
pub fn to_ascii(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|&b| {
            if (0x20..0x7f).contains(&b) {
                b as char
            } else {
                '.'
            }
        })
        .collect()
}

/// How closely after a marker a frame must arrive to be considered related.
///
/// 2 seconds is generous on purpose. A tighter window would hide slow
/// responses; a looser one would sweep in unrelated periodic traffic.
pub const CORRELATION_WINDOW_MS: u128 = 2000;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct FramePattern {
    pub hex: String,
    pub length: usize,
    pub occurrences: usize,
    /// Actions this pattern was seen shortly after, with counts. Presence here
    /// means temporal proximity only.
    pub seen_after: BTreeMap<String, usize>,
    /// Times it appeared with no marker in the preceding window — i.e. the
    /// device volunteered it unprompted.
    pub unprompted: usize,
}

/// Groups identical frames and reports which marked actions preceded them.
pub fn analyse(events: &[Event]) -> Vec<FramePattern> {
    let markers: Vec<(u128, Action)> = events
        .iter()
        .filter_map(|e| match e {
            Event::Marker { at_ms, action, .. } => Some((*at_ms, *action)),
            _ => None,
        })
        .collect();

    let mut by_hex: BTreeMap<String, FramePattern> = BTreeMap::new();

    for e in events {
        let Event::Frame {
            at_ms, hex, length, ..
        } = e
        else {
            continue;
        };

        let entry = by_hex.entry(hex.clone()).or_insert_with(|| FramePattern {
            hex: hex.clone(),
            length: *length,
            occurrences: 0,
            seen_after: BTreeMap::new(),
            unprompted: 0,
        });
        entry.occurrences += 1;

        // The most recent marker within the window, if any.
        let preceding = markers
            .iter()
            .filter(|(m, _)| *m <= *at_ms && at_ms - *m <= CORRELATION_WINDOW_MS)
            .max_by_key(|(m, _)| *m);

        match preceding {
            Some((_, action)) => {
                *entry
                    .seen_after
                    .entry(action.label().to_string())
                    .or_insert(0) += 1;
            }
            None => entry.unprompted += 1,
        }
    }

    let mut out: Vec<_> = by_hex.into_values().collect();
    out.sort_by(|a, b| b.occurrences.cmp(&a.occurrences));
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frame(at_ms: u128, hex: &str) -> Event {
        Event::Frame {
            at_ms,
            timestamp: String::new(),
            direction: "device-to-host",
            service_uuid: String::new(),
            length: 2,
            hex: hex.to_string(),
            ascii: String::new(),
        }
    }

    fn marker(at_ms: u128, action: Action) -> Event {
        Event::Marker {
            at_ms,
            timestamp: String::new(),
            action,
        }
    }

    #[test]
    fn hex_and_ascii_rendering() {
        assert_eq!(to_hex(&[0x00, 0xAB, 0xFF]), "00 AB FF");
        assert_eq!(to_ascii(&[0x41, 0x00, 0x42]), "A.B");
    }

    #[test]
    fn groups_identical_frames() {
        let events = vec![frame(0, "01 02"), frame(100, "01 02"), frame(200, "03 04")];
        let patterns = analyse(&events);
        assert_eq!(patterns.len(), 2);
        assert_eq!(patterns[0].hex, "01 02");
        assert_eq!(patterns[0].occurrences, 2);
    }

    #[test]
    fn correlates_a_frame_with_a_preceding_marker() {
        let events = vec![marker(1000, Action::AncAware), frame(1200, "AA BB")];
        let patterns = analyse(&events);
        assert_eq!(patterns[0].seen_after.get("ANC_AWARE"), Some(&1));
        assert_eq!(patterns[0].unprompted, 0);
    }

    /// A frame arriving long after a marker is not related to it.
    #[test]
    fn frames_outside_the_window_are_unprompted() {
        let events = vec![
            marker(0, Action::AncQuiet),
            frame(CORRELATION_WINDOW_MS + 500, "AA BB"),
        ];
        let patterns = analyse(&events);
        assert!(patterns[0].seen_after.is_empty());
        assert_eq!(patterns[0].unprompted, 1);
    }

    /// A frame before any marker cannot have been caused by one.
    #[test]
    fn frames_preceding_all_markers_are_unprompted() {
        let events = vec![frame(100, "AA BB"), marker(1000, Action::AncQuiet)];
        let patterns = analyse(&events);
        assert_eq!(patterns[0].unprompted, 1);
    }

    #[test]
    fn attributes_to_the_nearest_preceding_marker() {
        let events = vec![
            marker(0, Action::AncQuiet),
            marker(900, Action::AncAware),
            frame(1000, "AA BB"),
        ];
        let patterns = analyse(&events);
        assert_eq!(patterns[0].seen_after.get("ANC_AWARE"), Some(&1));
        assert_eq!(patterns[0].seen_after.get("ANC_QUIET"), None);
    }

    #[test]
    fn key_codes_map_to_actions() {
        assert_eq!(Action::from_key("1"), Some(Action::AncQuiet));
        assert_eq!(Action::from_key("U"), Some(Action::VolumeUp));
        assert_eq!(Action::from_key("zzz"), None);
    }
}
