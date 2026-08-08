//! Small shared helpers.

use sha2::{Digest, Sha256};

/// Per-installation salt for device id hashing.
///
/// Generated once and stored alongside the settings database. A fixed salt
/// would let anyone correlate an exported report back to a specific Bluetooth
/// address by hashing candidates; a per-install salt prevents that.
static SALT: std::sync::OnceLock<String> = std::sync::OnceLock::new();

pub fn set_salt(salt: String) {
    let _ = SALT.set(salt);
}

fn salt() -> &'static str {
    SALT.get().map(String::as_str).unwrap_or("uninitialised-salt")
}

/// A stable, opaque identifier derived from a raw device identifier.
///
/// The raw Bluetooth address or PnP instance id never leaves the native layer.
/// This is what appears in the UI and in exported diagnostics reports, so a
/// shared report cannot be used to identify or track the hardware.
pub fn stable_id(raw: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(salt().as_bytes());
    hasher.update(b"\x00");
    hasher.update(raw.as_bytes());
    let digest = hasher.finalize();
    // 80 bits is plenty to distinguish the handful of devices a person pairs,
    // and short enough to display.
    digest[..10]
        .iter()
        .map(|b| format!("{b:02x}"))
        .collect::<String>()
}

/// Generates a fresh random salt.
pub fn generate_salt() -> String {
    // Not cryptographic key material; uniqueness per installation is enough.
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stable_id_is_deterministic_within_an_installation() {
        let a = stable_id("BTHENUM\\DEV_AABBCCDDEEFF");
        let b = stable_id("BTHENUM\\DEV_AABBCCDDEEFF");
        assert_eq!(a, b);
    }

    #[test]
    fn different_devices_get_different_ids() {
        assert_ne!(
            stable_id("BTHENUM\\DEV_AABBCCDDEEFF"),
            stable_id("BTHENUM\\DEV_112233445566")
        );
    }

    /// The whole point: the raw address must not be recoverable or visible.
    #[test]
    fn stable_id_does_not_leak_the_raw_identifier() {
        let raw = "BTHENUM\\DEV_AABBCCDDEEFF";
        let id = stable_id(raw);
        assert!(!id.contains("AABBCCDDEEFF"));
        assert!(!id.to_uppercase().contains("BTHENUM"));
        assert_eq!(id.len(), 20);
    }

    #[test]
    fn salts_differ_between_generations() {
        assert_ne!(generate_salt(), generate_salt());
    }
}
