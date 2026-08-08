//! GATT service identification from Windows PnP nodes.
//!
//! Windows creates a child device node per discovered GATT service, named
//! `BTHLEDEVICE\{service-uuid}_...`. That means the service list of a paired BLE
//! device is readable without opening a GATT session — which fits the read-only
//! posture exactly, and is enough to reveal whether a device exposes standard
//! services, vendor-specific ones, or both.
//!
//! What this *cannot* do is enumerate characteristics; that needs a real GATT
//! session via WinRT. The report says so rather than implying the service list
//! is the whole picture.

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct GattService {
    pub uuid: String,
    /// Assigned-number name when the UUID is a standard one, else `None`.
    pub known_name: Option<String>,
    /// True when the UUID is outside the Bluetooth SIG base range, i.e. a
    /// vendor-defined service. These are the interesting ones.
    pub vendor_specific: bool,
}

/// The Bluetooth SIG base UUID suffix. Any UUID ending with this is a
/// 16-bit assigned number rather than a vendor-defined service.
const SIG_BASE_SUFFIX: &str = "-0000-1000-8000-00805F9B34FB";

/// Standard services worth naming in a report. Not exhaustive — only those
/// plausibly relevant to headphones.
fn known_service_name(short: &str) -> Option<&'static str> {
    Some(match short {
        "1800" => "Generic Access",
        "1801" => "Generic Attribute",
        "180A" => "Device Information",
        "180F" => "Battery Service",
        "1812" => "Human Interface Device",
        "1843" => "Audio Input Control",
        "1844" => "Volume Control",
        "1845" => "Volume Offset Control",
        "1846" => "Coordinated Set Identification",
        "1848" => "Media Control",
        "184E" => "Audio Stream Control",
        "184F" => "Broadcast Audio Scan",
        "1850" => "Published Audio Capabilities",
        "1853" => "Common Audio",
        "FDE2" => "Google/Fast Pair (member service)",
        "FE2C" => "Google Fast Pair",
        _ => return None,
    })
}

/// Parses a GATT service UUID out of a `BTHLEDEVICE` instance id.
///
/// Example input:
/// `BTHLEDEVICE\{0000180F-0000-1000-8000-00805F9B34FB}_DEV_VID&...\8&30736FC1&0&0020`
pub fn parse_service(instance_id: &str) -> Option<GattService> {
    let upper = instance_id.to_uppercase();
    if !upper.starts_with("BTHLEDEVICE\\") {
        return None;
    }

    let start = upper.find('{')? + 1;
    let end = upper[start..].find('}')? + start;
    let uuid = &upper[start..end];

    // A UUID is 36 characters. Anything else is not a service node.
    if uuid.len() != 36 {
        return None;
    }

    let is_sig = uuid.ends_with(SIG_BASE_SUFFIX);
    let known_name = if is_sig {
        // First group is 0000XXXX for assigned numbers.
        uuid.get(4..8).and_then(known_service_name).map(str::to_string)
    } else {
        None
    };

    Some(GattService {
        uuid: uuid.to_string(),
        known_name,
        vendor_specific: !is_sig,
    })
}

/// The device address embedded in a `BTHLEDEVICE` instance id, used to group
/// service nodes under the device they belong to.
pub fn owning_address(instance_id: &str) -> Option<String> {
    let upper = instance_id.to_uppercase();
    // Trailing 12 hex characters before the final backslash section.
    let head = upper.split('\\').nth(1)?;
    let tail = head.rsplit('_').next()?;
    if tail.len() == 12 && tail.chars().all(|c| c.is_ascii_hexdigit()) {
        Some(tail.to_string())
    } else {
        None
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Instance ids below were captured from the development machine, so these
    // tests exercise real Windows formatting rather than invented strings.

    #[test]
    fn parses_standard_battery_service() {
        let s = parse_service(
            "BTHLEDEVICE\\{0000180F-0000-1000-8000-00805F9B34FB}_DEV_VID&0017EF_PID&6134_REV&0026_79C657FDB4BC\\8&30736FC1&0&0020",
        )
        .unwrap();
        assert_eq!(s.known_name.as_deref(), Some("Battery Service"));
        assert!(!s.vendor_specific);
    }

    #[test]
    fn parses_device_information_service() {
        let s = parse_service(
            "BTHLEDEVICE\\{0000180A-0000-1000-8000-00805F9B34FB}_DEV_VID&0017EF_PID&6134_REV&0026_79C657FDB4BC\\8&30736FC1&0&000E",
        )
        .unwrap();
        assert_eq!(s.known_name.as_deref(), Some("Device Information"));
        assert!(!s.vendor_specific);
    }

    /// The whole point of the tool: spotting non-standard services. This UUID
    /// is a real vendor service observed on the development machine.
    #[test]
    fn flags_vendor_specific_service() {
        let s = parse_service(
            "BTHLEDEVICE\\{594A34FC-31DB-11EA-978F-2E728CE88125}_34F043C9E0F6\\8&2D7DA683&0&0086",
        )
        .unwrap();
        assert!(s.vendor_specific);
        assert!(s.known_name.is_none());
    }

    #[test]
    fn ignores_non_service_nodes() {
        assert!(parse_service("BTHLE\\DEV_79C657FDB4BC\\7&1E36B139&0&79C657FDB4BC").is_none());
        assert!(parse_service(
            "BTHENUM\\DEV_E458BCF9F02E\\7&78167D1&0&BLUETOOTHDEVICE_E458BCF9F02E"
        )
        .is_none());
    }

    #[test]
    fn extracts_owning_address() {
        assert_eq!(
            owning_address(
                "BTHLEDEVICE\\{00001801-0000-1000-8000-00805F9B34FB}_34F043C9E0F6\\8&2D7DA683&0&0001"
            )
            .as_deref(),
            Some("34F043C9E0F6")
        );
    }
}
