//! The hardware-backed device.
//!
//! What this currently does is deliberately narrow, and the narrowness is the
//! point. It reports what standard, documented Windows interfaces can actually
//! tell us about the headphones:
//!
//!   * that the device exists and is paired          (PnP enumeration)
//!   * whether Windows considers it connected        (PnP property)
//!   * its battery level, *if* the device populates  (PnP battery property)
//!     the property Windows Settings itself reads
//!
//! What it does **not** do is pretend to control noise cancellation or EQ.
//! Those live behind a Bose vendor protocol that has not been verified against
//! physical hardware, so the mutating methods return `Unsupported` with a
//! reason rather than sending speculative bytes at the headphones. When the
//! protocol work in `docs/protocol-notes.md` produces verified results, this is
//! the single file that changes.

use crate::device::capability::{CapabilityKey, CapabilitySet, HardwareProof, Mechanism};
use crate::device::command::CommandOutcome;
use crate::device::state::{
    BatteryReading, BatterySource, ConnectionState, DeviceIdentity, DeviceSnapshot, DeviceSource,
    EqSettings, EqState, NoiseControlMode, NoiseControlState, Transport,
};
use crate::device::traits::BoseDevice;
use crate::device::{now_rfc3339, DeviceError, DeviceResult};
use async_trait::async_trait;
use parking_lot::RwLock;

/// Reason text reused wherever a vendor-protocol feature is requested.
const NO_VERIFIED_VENDOR_PROTOCOL: &str = "No Bose vendor protocol has been verified against this \
     device yet. This build will not send speculative commands to your headphones. See \
     docs/protocol-notes.md.";

#[derive(Debug, Clone, Default)]
struct Observed {
    instance_id: Option<String>,
    name: Option<String>,
    connected: bool,
    battery: Option<u8>,
    /// Set once we have actually read a battery value from the physical
    /// device, which is genuine hardware evidence for the battery capability.
    battery_ever_read: bool,
}

pub struct RealBoseDevice {
    /// Optional explicit target. When `None`, the first device whose Windows
    /// friendly name matches a Bose hint is used.
    preferred_instance_id: Option<String>,
    observed: RwLock<Observed>,
}

impl Default for RealBoseDevice {
    fn default() -> Self {
        Self::new(None)
    }
}

impl RealBoseDevice {
    pub fn new(preferred_instance_id: Option<String>) -> Self {
        Self {
            preferred_instance_id,
            observed: RwLock::new(Observed::default()),
        }
    }

    /// Locates the target device and refreshes the cached observation.
    ///
    /// Returns `Ok(false)` when no matching device is present — that is a
    /// normal state (headphones off or unpaired), not an error.
    #[cfg(windows)]
    fn refresh(&self) -> DeviceResult<bool> {
        use crate::bluetooth::pnp;

        let radio = crate::bluetooth::radio::availability();
        if !radio.radio_present {
            return Err(DeviceError::BluetoothUnavailable);
        }

        let devices = pnp::enumerate_bluetooth_devices();

        let matched = devices
            .into_iter()
            .filter(|d| d.is_top_level())
            .find(|d| match &self.preferred_instance_id {
                Some(want) => &d.instance_id == want,
                // Vendor id first: the friendly name is user-editable and the
                // test device is renamed, so name matching alone finds nothing.
                None => super::is_bose_device(
                    d.vendor_id,
                    d.friendly_name.as_deref().unwrap_or_default(),
                ),
            });

        let Some(dev) = matched else {
            *self.observed.write() = Observed::default();
            return Ok(false);
        };

        let mut obs = self.observed.write();
        obs.instance_id = Some(dev.instance_id.clone());
        obs.name = dev.friendly_name.clone();
        obs.connected = dev.is_connected.unwrap_or(false);
        obs.battery = dev.battery_percent;
        if dev.battery_percent.is_some() {
            obs.battery_ever_read = true;
        }
        Ok(true)
    }

    #[cfg(not(windows))]
    fn refresh(&self) -> DeviceResult<bool> {
        Err(DeviceError::Platform(
            "the real device backend is only implemented for Windows".to_string(),
        ))
    }

    fn build_capabilities(&self, obs: &Observed) -> CapabilitySet {
        let mut caps = CapabilitySet::all_unknown(
            "Not yet investigated on this device. Run Diagnostics to interrogate it.",
        );

        if obs.instance_id.is_none() {
            return caps;
        }

        // Battery. If we have actually read a value off the physical device,
        // that is hardware evidence that the read path works end to end.
        // If the property is simply absent, we say so rather than guessing.
        if obs.battery_ever_read {
            let proof = HardwareProof::observed_passively(
                CapabilityKey::Battery,
                format!(
                    "Windows PnP battery property returned a level for this device{}",
                    obs.battery
                        .map(|b| format!(" (currently {b}%)"))
                        .unwrap_or_default()
                ),
            );
            let _ = caps
                .get_mut(CapabilityKey::Battery)
                .verify_with_hardware(&proof, Mechanism::WindowsPnp);
        } else if obs.connected {
            let _ = caps.get_mut(CapabilityKey::Battery).mark_supported(
                Mechanism::WindowsPnp,
                "The device is connected but has not populated the Windows battery property. \
                 It may report battery only while audio is streaming, or not at all.",
            );
        }

        // Device identity is readable straight from Windows for any paired
        // device, and we have just read it.
        let proof = HardwareProof::observed_passively(
            CapabilityKey::DeviceSettings,
            "device identity read from Windows PnP",
        );
        let _ = caps
            .get_mut(CapabilityKey::DeviceSettings)
            .verify_with_hardware(&proof, Mechanism::WindowsPnp);

        // Everything requiring the Bose vendor protocol stays UNKNOWN until
        // discovery has actually interrogated the device. Not "supported" —
        // we have no evidence at all yet.
        for key in [
            CapabilityKey::NoiseControl,
            CapabilityKey::AwareMode,
            CapabilityKey::CustomNoiseControl,
            CapabilityKey::Equalizer,
            CapabilityKey::Multipoint,
            CapabilityKey::FirmwareVersion,
            CapabilityKey::AutoOff,
            CapabilityKey::VoicePrompts,
            CapabilityKey::Sidetone,
            CapabilityKey::DeviceRename,
        ] {
            let _ = caps.get_mut(key).mark_supported(
                Mechanism::None,
                "Requires a Bose vendor protocol that has not been verified on this device.",
            );
            // mark_supported would overstate it; reset to unknown explicitly.
            *caps.get_mut(key) = crate::device::Capability::unknown(
                key,
                "Requires a Bose vendor protocol that has not been investigated on this device yet.",
            );
        }

        caps
    }

    fn identity_from(obs: &Observed) -> Option<DeviceIdentity> {
        let instance_id = obs.instance_id.as_ref()?;
        Some(DeviceIdentity {
            name: obs
                .name
                .clone()
                .unwrap_or_else(|| "Bose device".to_string()),
            id: crate::util::stable_id(instance_id),
            // The raw instance id is kept out of the snapshot handed to the UI
            // so it cannot leak into an exported report or a screenshot.
            instance_id: None,
            manufacturer: Some("Bose".to_string()),
            model_number: None,
            firmware_version: None,
            serial_number: None,
        })
    }
}

#[async_trait]
impl BoseDevice for RealBoseDevice {
    fn source(&self) -> DeviceSource {
        DeviceSource::Real
    }

    async fn capabilities(&self) -> CapabilitySet {
        let _ = self.refresh();
        let obs = self.observed.read().clone();
        self.build_capabilities(&obs)
    }

    async fn snapshot(&self) -> DeviceResult<DeviceSnapshot> {
        let found = self.refresh()?;
        let obs = self.observed.read().clone();
        let now = now_rfc3339();

        let connection = if !found {
            ConnectionState::Disconnected
        } else if obs.connected {
            ConnectionState::Connected
        } else {
            ConnectionState::Disconnected
        };

        Ok(DeviceSnapshot {
            source: DeviceSource::Real,
            connection,
            transport: if obs.connected {
                Transport::BluetoothClassic
            } else {
                Transport::None
            },
            identity: Self::identity_from(&obs),
            capabilities: self.build_capabilities(&obs),
            battery: obs.battery.map(|percent| BatteryReading {
                percent,
                source: BatterySource::WindowsPnp,
                charging: None,
                read_at: now.clone(),
            }),
            // Never fabricated. We have no verified way to read these yet.
            noise_control: None,
            equalizer: None,
            windows_audio: None,
            last_error: None,
            updated_at: now,
        })
    }

    async fn connect(&self) -> DeviceResult<CommandOutcome> {
        // Windows owns Bluetooth audio connection policy. We can observe the
        // connection, but initiating one for an already-paired A2DP device is
        // not something this application should force. Report honestly.
        let found = self.refresh()?;
        if !found {
            return Err(DeviceError::Unreachable);
        }
        let connected = self.observed.read().connected;
        if connected {
            Ok(CommandOutcome::applied())
        } else {
            Ok(CommandOutcome::sent_unverified(
                "Windows manages the audio connection for paired headphones. \
                 Power the headphones on and Windows will connect them automatically.",
            ))
        }
    }

    async fn disconnect(&self) -> DeviceResult<CommandOutcome> {
        Ok(CommandOutcome::unsupported(
            "Disconnecting a paired audio device is managed by Windows. \
             Use Windows Settings > Bluetooth & devices.",
        ))
    }

    async fn device_info(&self) -> DeviceResult<Option<DeviceIdentity>> {
        self.refresh()?;
        let obs = self.observed.read().clone();
        Ok(Self::identity_from(&obs))
    }

    async fn battery(&self) -> DeviceResult<Option<BatteryReading>> {
        self.refresh()?;
        let obs = self.observed.read().clone();
        Ok(obs.battery.map(|percent| BatteryReading {
            percent,
            source: BatterySource::WindowsPnp,
            charging: None,
            read_at: now_rfc3339(),
        }))
    }

    async fn noise_control(&self) -> DeviceResult<Option<NoiseControlState>> {
        // We genuinely do not know. `None` says exactly that; it is not an
        // error, and it is certainly not a default of "Quiet".
        Ok(None)
    }

    async fn set_noise_control(&self, _mode: NoiseControlMode) -> DeviceResult<CommandOutcome> {
        Ok(CommandOutcome::unsupported(NO_VERIFIED_VENDOR_PROTOCOL))
    }

    async fn equalizer(&self) -> DeviceResult<Option<EqState>> {
        Ok(None)
    }

    async fn set_equalizer(&self, _settings: EqSettings) -> DeviceResult<CommandOutcome> {
        Ok(CommandOutcome::unsupported(NO_VERIFIED_VENDOR_PROTOCOL))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// The defining property of the current real backend: it refuses to
    /// pretend. Until a protocol is verified, mutating vendor features must
    /// report Unsupported rather than silently doing nothing and returning OK.
    #[tokio::test]
    async fn vendor_features_report_unsupported_not_success() {
        let dev = RealBoseDevice::new(Some("NON_EXISTENT_DEVICE".to_string()));

        let outcome = dev.set_noise_control(NoiseControlMode::Aware).await.unwrap();
        assert!(!outcome.is_confirmed());
        assert!(matches!(outcome, CommandOutcome::Unsupported { .. }));

        let outcome = dev.set_equalizer(EqSettings::flat()).await.unwrap();
        assert!(matches!(outcome, CommandOutcome::Unsupported { .. }));
    }

    #[tokio::test]
    async fn unknown_state_is_none_not_a_plausible_default() {
        let dev = RealBoseDevice::new(Some("NON_EXISTENT_DEVICE".to_string()));
        assert!(dev.noise_control().await.unwrap().is_none());
        assert!(dev.equalizer().await.unwrap().is_none());
    }

    #[test]
    fn capabilities_without_a_device_are_all_unknown() {
        let dev = RealBoseDevice::new(None);
        let caps = dev.build_capabilities(&Observed::default());
        assert_eq!(caps.verified_count(), 0);
        for cap in caps.iter() {
            assert_eq!(cap.status, crate::device::CapabilityStatus::Unknown);
        }
    }

    /// A device that never reported battery must not be described as having
    /// battery support.
    #[test]
    fn battery_capability_requires_an_actual_reading() {
        let dev = RealBoseDevice::new(None);
        let obs = Observed {
            instance_id: Some("BTHENUM\\DEV_TEST".into()),
            name: Some("Bose QuietComfort".into()),
            connected: true,
            battery: None,
            battery_ever_read: false,
        };
        let caps = dev.build_capabilities(&obs);
        let battery = caps.get(CapabilityKey::Battery);
        assert!(!battery.hardware_verified);
        assert_ne!(battery.status, crate::device::CapabilityStatus::Verified);
    }

    #[test]
    fn battery_capability_is_verified_once_actually_read() {
        let dev = RealBoseDevice::new(None);
        let obs = Observed {
            instance_id: Some("BTHENUM\\DEV_TEST".into()),
            name: Some("Bose QuietComfort".into()),
            connected: true,
            battery: Some(72),
            battery_ever_read: true,
        };
        let caps = dev.build_capabilities(&obs);
        let battery = caps.get(CapabilityKey::Battery);
        assert!(battery.hardware_verified);
        assert_eq!(battery.status, crate::device::CapabilityStatus::Verified);
    }

    /// Vendor-protocol features must stay UNKNOWN even when a Bose device is
    /// present and connected. Presence of the device is not evidence about ANC.
    #[test]
    fn vendor_capabilities_stay_unknown_even_with_device_present() {
        let dev = RealBoseDevice::new(None);
        let obs = Observed {
            instance_id: Some("BTHENUM\\DEV_TEST".into()),
            name: Some("Bose QuietComfort".into()),
            connected: true,
            battery: Some(72),
            battery_ever_read: true,
        };
        let caps = dev.build_capabilities(&obs);
        for key in [
            CapabilityKey::NoiseControl,
            CapabilityKey::AwareMode,
            CapabilityKey::Equalizer,
        ] {
            assert_eq!(
                caps.get(key).status,
                crate::device::CapabilityStatus::Unknown,
                "{key:?} must remain unknown until investigated"
            );
        }
    }
}
