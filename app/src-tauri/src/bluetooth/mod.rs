//! Windows Bluetooth access.
//!
//! Scope note: everything currently implemented here is **read-only**. There is
//! no code path in this module that writes to a Bluetooth device. That is a
//! deliberate constraint until a vendor protocol has been verified against
//! physical hardware — see `docs/protocol-notes.md`.

#[cfg(windows)]
pub mod pnp;

#[cfg(windows)]
pub mod radio;

use serde::{Deserialize, Serialize};

/// A Bluetooth device as discovered through Windows, before any Bose-specific
/// interpretation is applied.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DiscoveredDevice {
    /// Opaque, salted hash of the instance id — safe to include in an exported
    /// report. The raw instance id stays in memory only.
    pub id: String,
    pub name: String,
    pub transport: DiscoveredTransport,
    pub connected: Option<bool>,
    /// Battery as reported by the Windows PnP property, when present.
    pub battery_percent: Option<u8>,
    /// Whether the name matches a Bose hint. A hint only — never an identification.
    pub looks_like_bose: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DiscoveredTransport {
    Classic,
    LowEnergy,
    Unknown,
}

/// Whether the machine has a usable Bluetooth radio at all.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct BluetoothAvailability {
    pub radio_present: bool,
    pub radio_enabled: bool,
    pub detail: String,
}

impl BluetoothAvailability {
    pub fn unavailable(detail: impl Into<String>) -> Self {
        Self {
            radio_present: false,
            radio_enabled: false,
            detail: detail.into(),
        }
    }
}
