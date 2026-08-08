//! Report generation.
//!
//! Reports are meant to be shared — attached to a bug report, pasted into a
//! discussion. So they must not contain anything that identifies the hardware.
//! Bluetooth addresses are replaced with salted hashes, and the salt is
//! regenerated per run, so two reports from the same machine cannot even be
//! correlated with each other.

use crate::gatt::GattService;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::path::{Path, PathBuf};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct DeviceRecord {
    /// Salted hash. Never a Bluetooth address.
    pub id: String,
    pub name: String,
    pub transport: String,
    pub connected: Option<bool>,
    pub battery_percent: Option<u8>,
    pub looks_like_bose: bool,
    pub gatt_services: Vec<GattService>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Report {
    pub tool: String,
    pub version: String,
    pub generated_at: String,
    /// What this report does and does not contain. Included in the file so a
    /// reader does not have to take the tool's word for it.
    pub scope: ReportScope,
    pub radio_nodes: usize,
    pub devices: Vec<DeviceRecord>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ReportScope {
    pub read_only: bool,
    pub bytes_written_to_devices: usize,
    pub contains_bluetooth_addresses: bool,
    pub notes: Vec<String>,
}

impl Default for ReportScope {
    fn default() -> Self {
        Self {
            read_only: true,
            bytes_written_to_devices: 0,
            contains_bluetooth_addresses: false,
            notes: vec![
                "This tool is read-only. It sent nothing to any Bluetooth device.".to_string(),
                "Device identifiers are per-run salted hashes, not Bluetooth addresses."
                    .to_string(),
                "Device NAMES are included as Windows reports them, and people often name \
                 devices after themselves. Check the names below before sharing this report, \
                 or re-run with --redact-names."
                    .to_string(),
                "GATT services are read from Windows PnP nodes created during pairing. \
                 Characteristic enumeration requires an active GATT session and is NOT \
                 included in this report."
                    .to_string(),
                "Absence of a service here means Windows did not record it, which is not \
                 proof the device does not expose it."
                    .to_string(),
            ],
        }
    }
}

impl Report {
    pub fn new(radio_nodes: usize, devices: Vec<DeviceRecord>) -> Self {
        Self {
            tool: "bose-discovery".to_string(),
            version: env!("CARGO_PKG_VERSION").to_string(),
            generated_at: now_rfc3339(),
            scope: ReportScope::default(),
            radio_nodes,
            devices,
        }
    }

    pub fn to_json(&self) -> String {
        serde_json::to_string_pretty(self).unwrap_or_else(|e| format!("{{\"error\":\"{e}\"}}"))
    }

    pub fn to_text(&self) -> String {
        let mut out = String::new();
        out.push_str("BOSE QC CONTROL CENTER — DEVICE REPORT\n");
        out.push_str("======================================\n\n");
        out.push_str(&format!("Generated : {}\n", self.generated_at));
        out.push_str(&format!("Tool      : {} v{}\n", self.tool, self.version));
        out.push_str("Posture   : READ-ONLY (0 bytes written to any device)\n\n");

        out.push_str(&format!("Bluetooth radio nodes : {}\n", self.radio_nodes));
        out.push_str(&format!("Paired devices        : {}\n\n", self.devices.len()));

        let bose: Vec<_> = self.devices.iter().filter(|d| d.looks_like_bose).collect();
        out.push_str("BOSE NAME MATCHES\n-----------------\n");
        if bose.is_empty() {
            out.push_str("None. No paired device name matched a Bose hint.\n\n");
        } else {
            for d in &bose {
                out.push_str(&format!("  {}\n", d.name));
            }
            out.push('\n');
        }

        out.push_str("DEVICES\n-------\n");
        for d in &self.devices {
            out.push_str(&format!("\n  {}\n", d.name));
            out.push_str(&format!("    id         : {}\n", d.id));
            out.push_str(&format!("    transport  : {}\n", d.transport));
            out.push_str(&format!(
                "    connected  : {}\n",
                match d.connected {
                    Some(true) => "yes",
                    Some(false) => "no",
                    None => "unknown",
                }
            ));
            out.push_str(&format!(
                "    battery    : {}\n",
                match d.battery_percent {
                    Some(b) => format!("{b}%"),
                    None => "not reported".to_string(),
                }
            ));
            if d.gatt_services.is_empty() {
                out.push_str("    GATT       : none recorded by Windows\n");
            } else {
                out.push_str("    GATT services:\n");
                for s in &d.gatt_services {
                    let label = s.known_name.clone().unwrap_or_else(|| {
                        "VENDOR-SPECIFIC (investigate)".to_string()
                    });
                    out.push_str(&format!("      {}  {}\n", s.uuid, label));
                }
            }
        }

        out.push_str("\nNOTES\n-----\n");
        for note in &self.scope.notes {
            out.push_str(&format!("  - {note}\n"));
        }
        out
    }

    /// Writes both report formats. Returns the paths written.
    pub fn write_to_disk(&self, dir: &Path) -> std::io::Result<(PathBuf, PathBuf)> {
        let json_path = dir.join("device-report.json");
        let txt_path = dir.join("device-report.txt");
        std::fs::write(&json_path, self.to_json())?;
        std::fs::write(&txt_path, self.to_text())?;
        Ok((json_path, txt_path))
    }
}

/// A fresh salt for this run.
pub fn session_salt() -> String {
    let mut hasher = Sha256::new();
    hasher.update(
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_nanos().to_le_bytes())
            .unwrap_or_default(),
    );
    hasher.update(std::process::id().to_le_bytes());
    format!("{:x}", hasher.finalize())
}

/// Replaces a device name with a non-identifying description.
///
/// Keeps the information that matters for diagnosis — whether it looks like a
/// Bose device, and roughly what kind of thing it is — while dropping anything
/// personal. "Jaisal's S24 Ultra" becomes "[redacted device]".
pub fn redact_name(name: &str, looks_like_bose: bool) -> String {
    if looks_like_bose {
        // Preserved deliberately: the whole point of the report is to identify
        // the Bose device, and the model matters for protocol work.
        return name.to_string();
    }
    "[redacted device]".to_string()
}

pub fn stable_id(salt: &str, raw: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(salt.as_bytes());
    hasher.update(b"\x00");
    hasher.update(raw.as_bytes());
    hasher.finalize()[..10]
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect()
}

fn now_rfc3339() -> String {
    time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_else(|_| "unknown".to_string())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample() -> Report {
        Report::new(
            1,
            vec![DeviceRecord {
                id: stable_id("salt", "BTHENUM\\DEV_AABBCCDDEEFF"),
                name: "Bose QuietComfort".to_string(),
                transport: "bluetooth-classic".to_string(),
                connected: Some(true),
                battery_percent: Some(72),
                looks_like_bose: true,
                gatt_services: vec![GattService {
                    uuid: "0000180F-0000-1000-8000-00805F9B34FB".to_string(),
                    known_name: Some("Battery Service".to_string()),
                    vendor_specific: false,
                }],
            }],
        )
    }

    /// The property that makes a report safe to share.
    #[test]
    fn report_contains_no_bluetooth_address() {
        let report = sample();
        let json = report.to_json();
        let text = report.to_text();
        assert!(!json.contains("AABBCCDDEEFF"));
        assert!(!text.contains("AABBCCDDEEFF"));
    }

    #[test]
    fn report_declares_its_read_only_posture() {
        let report = sample();
        assert!(report.scope.read_only);
        assert_eq!(report.scope.bytes_written_to_devices, 0);
        assert!(!report.scope.contains_bluetooth_addresses);
        assert!(report.to_text().contains("READ-ONLY"));
    }

    #[test]
    fn json_round_trips() {
        let report = sample();
        let back: Report = serde_json::from_str(&report.to_json()).unwrap();
        assert_eq!(report, back);
    }

    #[test]
    fn text_report_flags_vendor_services_for_investigation() {
        let mut report = sample();
        report.devices[0].gatt_services.push(GattService {
            uuid: "594A34FC-31DB-11EA-978F-2E728CE88125".to_string(),
            known_name: None,
            vendor_specific: true,
        });
        assert!(report.to_text().contains("VENDOR-SPECIFIC"));
    }

    #[test]
    fn redaction_drops_personal_names_but_keeps_bose_models() {
        // A device named after its owner must not survive redaction.
        assert_eq!(redact_name("Jaisal's S24 Ultra", false), "[redacted device]");
        // The Bose device is the subject of the report, so it is kept.
        assert_eq!(
            redact_name("Bose QuietComfort Headphones", true),
            "Bose QuietComfort Headphones"
        );
    }

    #[test]
    fn salts_differ_between_runs() {
        assert_ne!(session_salt(), session_salt());
    }

    #[test]
    fn same_salt_and_input_gives_same_id() {
        assert_eq!(stable_id("s", "device"), stable_id("s", "device"));
        assert_ne!(stable_id("s1", "device"), stable_id("s2", "device"));
    }
}
