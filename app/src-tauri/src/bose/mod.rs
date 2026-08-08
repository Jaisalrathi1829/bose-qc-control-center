//! Bose-specific device implementations.
//!
//! `mock` is the simulated device used for development and regression tests.
//!
//! `real` is the hardware-backed device. It is deliberately conservative: at
//! present it reports what standard Windows interfaces can actually tell us
//! about the headphones, and nothing more. No vendor protocol is implemented,
//! because none has been verified against the user's physical device yet. See
//! `docs/protocol-notes.md` for the current state of that investigation.

pub mod mock;
pub mod real;

pub use mock::{MockBehaviour, MockBoseDevice};
pub use real::RealBoseDevice;

/// Name fragments that identify a Bose device from a Windows friendly name.
///
/// Matching is case-insensitive and substring-based because Windows friendly
/// names vary by pairing method and firmware ("Bose QuietComfort Headphones",
/// "LE_Bose QC Ultra", a user-renamed device, and so on).
pub const BOSE_NAME_HINTS: &[&str] = &[
    "bose",
    "quietcomfort",
    "quiet comfort",
    "qc45",
    "qc35",
    "qc ultra",
];

/// Whether a Windows friendly name plausibly belongs to a Bose device.
///
/// This is a *hint*, not an identification. A positive match means "worth
/// interrogating", never "this is the user's QC".
pub fn looks_like_bose(friendly_name: &str) -> bool {
    let lowered = friendly_name.to_lowercase();
    BOSE_NAME_HINTS.iter().any(|hint| lowered.contains(hint))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognises_plausible_bose_names() {
        assert!(looks_like_bose("Bose QuietComfort Headphones"));
        assert!(looks_like_bose("LE_Bose QC Ultra Headphones"));
        assert!(looks_like_bose("BOSE QC45"));
        assert!(looks_like_bose("quietcomfort 45"));
    }

    #[test]
    fn does_not_match_unrelated_devices() {
        assert!(!looks_like_bose("Jaisal's S24 Ultra"));
        assert!(!looks_like_bose("Legion M600s Mouse"));
        assert!(!looks_like_bose("ZEB-DUKE"));
        assert!(!looks_like_bose("Speakers (Realtek(R) Audio)"));
    }

    /// "Ultra" alone must not match — several unrelated devices use it, as the
    /// development machine's own paired phone demonstrates.
    #[test]
    fn ultra_alone_is_not_a_bose_hint() {
        assert!(!looks_like_bose("Galaxy S24 Ultra"));
    }
}
