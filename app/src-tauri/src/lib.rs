//! Bose QC Control Center — native layer.
//!
//! Architecture note: the frontend has no Bluetooth surface. It calls the
//! `#[tauri::command]` handlers below, each of which takes typed parameters
//! that are validated before any backend sees them. There is deliberately no
//! command that accepts a UUID, a characteristic handle, or a byte array.

pub mod audio;
pub mod bluetooth;
pub mod bose;
pub mod device;
pub mod util;

use bose::{MockBoseDevice, RealBoseDevice};
use device::capability::{CapabilityKey, HardwareProof, Mechanism};
use device::command::{CommandOutcome, DeviceCommand};
use device::state::{DeviceSnapshot, DeviceSource, WindowsAudioState};
use device::traits::BoseDevice;
use parking_lot::RwLock;
use std::sync::Arc;

/// Which backend the session is using.
pub struct AppState {
    device: RwLock<Arc<dyn BoseDevice>>,
}

impl AppState {
    pub fn new(source: DeviceSource) -> Self {
        Self {
            device: RwLock::new(Self::backend_for(source)),
        }
    }

    fn backend_for(source: DeviceSource) -> Arc<dyn BoseDevice> {
        match source {
            DeviceSource::Mock => Arc::new(MockBoseDevice::default()),
            DeviceSource::Real => Arc::new(RealBoseDevice::default()),
        }
    }

    pub fn current(&self) -> Arc<dyn BoseDevice> {
        self.device.read().clone()
    }

    pub fn switch_to(&self, source: DeviceSource) {
        *self.device.write() = Self::backend_for(source);
    }
}

/// Serializable error returned to the frontend.
#[derive(Debug, serde::Serialize)]
pub struct UiError {
    pub message: String,
    pub kind: String,
}

impl From<device::DeviceError> for UiError {
    fn from(e: device::DeviceError) -> Self {
        let kind = match &e {
            device::DeviceError::NotConnected => "not-connected",
            device::DeviceError::BluetoothUnavailable => "bluetooth-unavailable",
            device::DeviceError::Unreachable => "unreachable",
            device::DeviceError::Timeout(_) => "timeout",
            device::DeviceError::Unsupported { .. } => "unsupported",
            device::DeviceError::InvalidResponse(_) => "invalid-response",
            device::DeviceError::Rejected(_) => "rejected",
            device::DeviceError::Platform(_) => "platform",
            device::DeviceError::InvalidInput(_) => "invalid-input",
        };
        Self {
            message: e.to_string(),
            kind: kind.to_string(),
        }
    }
}

type UiResult<T> = Result<T, UiError>;

// --- Commands ---------------------------------------------------------------

/// Attaches Windows audio state to a device snapshot.
///
/// Audio is a system concern rather than a device-backend one: the endpoint
/// exists whether we are talking to a mock or to real headphones. Keeping it
/// out of the backends means the mock cannot fabricate a Windows volume, which
/// would be a particularly misleading thing to simulate.
///
/// Reading an endpoint successfully is genuine hardware evidence that the
/// volume path works, so the capability is promoted accordingly.
fn attach_windows_audio(mut snapshot: DeviceSnapshot) -> DeviceSnapshot {
    #[cfg(windows)]
    {
        match audio::default_output_endpoint() {
            Ok(Some(ep)) => {
                let proof = HardwareProof::observed_passively(
                    CapabilityKey::Volume,
                    format!("read Windows volume from endpoint {:?}", ep.name),
                );
                let _ = snapshot
                    .capabilities
                    .get_mut(CapabilityKey::Volume)
                    .verify_with_hardware(&proof, Mechanism::WindowsAudio);

                snapshot.windows_audio = Some(WindowsAudioState {
                    endpoint_name: ep.name,
                    endpoint_id: util::stable_id(&ep.id),
                    volume_percent: ep.volume_percent,
                    muted: ep.muted,
                    is_default_render: ep.is_default_render,
                    is_default_communications: ep.is_default_communications,
                    sample_rate_hz: ep.sample_rate_hz,
                    channels: ep.channels,
                });
            }
            Ok(None) => {
                let _ = snapshot
                    .capabilities
                    .get_mut(CapabilityKey::Volume)
                    .mark_supported(
                        Mechanism::WindowsAudio,
                        "Core Audio is available but no active output endpoint is present.",
                    );
            }
            Err(e) => {
                let _ = snapshot
                    .capabilities
                    .get_mut(CapabilityKey::Volume)
                    .mark_unsupported(format!("Core Audio unavailable: {e}"));
            }
        }
    }
    snapshot
}

/// Resolves the endpoint id to act on. `None` means the default endpoint.
///
/// The UI only ever sees salted hashes, so it cannot name a raw endpoint id —
/// which is deliberate. Volume commands therefore act on the default endpoint.
fn target_endpoint() -> Option<&'static str> {
    None
}

#[tauri::command]
async fn get_snapshot(state: tauri::State<'_, AppState>) -> UiResult<DeviceSnapshot> {
    let dev = state.current();
    Ok(attach_windows_audio(dev.snapshot().await?))
}

#[tauri::command]
fn list_audio_endpoints() -> UiResult<Vec<audio::AudioEndpoint>> {
    #[cfg(windows)]
    {
        Ok(audio::list_output_endpoints()?)
    }
    #[cfg(not(windows))]
    {
        Ok(Vec::new())
    }
}

#[tauri::command]
async fn get_device_source(state: tauri::State<'_, AppState>) -> UiResult<DeviceSource> {
    Ok(state.current().source())
}

#[tauri::command]
async fn set_device_source(
    state: tauri::State<'_, AppState>,
    source: DeviceSource,
) -> UiResult<DeviceSnapshot> {
    state.switch_to(source);
    let dev = state.current();
    Ok(attach_windows_audio(dev.snapshot().await?))
}

/// The single entry point for every device operation.
///
/// Validation happens here, once, for both mock and real backends — so the
/// mock exercises exactly the same checks the hardware path does.
#[tauri::command]
async fn execute_command(
    state: tauri::State<'_, AppState>,
    command: DeviceCommand,
) -> UiResult<CommandOutcome> {
    command.validate()?;

    let dev = state.current();
    let outcome = match command {
        DeviceCommand::Connect => dev.connect().await?,
        DeviceCommand::Disconnect => dev.disconnect().await?,
        DeviceCommand::Reconnect => {
            let _ = dev.disconnect().await;
            dev.connect().await?
        }
        DeviceCommand::SetNoiseControl { mode } => dev.set_noise_control(mode).await?,
        DeviceCommand::SetEqualizer { settings } => dev.set_equalizer(settings).await?,

        DeviceCommand::RefreshSnapshot
        | DeviceCommand::ReadBattery
        | DeviceCommand::ReadNoiseControl
        | DeviceCommand::ReadEqualizer
        | DeviceCommand::ReadDeviceInfo => CommandOutcome::applied(),

        DeviceCommand::SetNoiseControlLevel { .. } => CommandOutcome::unsupported(
            "Continuous noise-control level requires a verified vendor protocol.",
        ),

        // Windows system volume. Set, then re-read the endpoint and only
        // report success if it actually reports the requested value.
        DeviceCommand::SetSystemVolume { percent } => {
            #[cfg(windows)]
            {
                let actual = audio::set_volume(target_endpoint(), percent)?;
                if actual == percent {
                    CommandOutcome::applied()
                } else {
                    CommandOutcome::sent_unverified(format!(
                        "Endpoint reports {actual}% after requesting {percent}%."
                    ))
                }
            }
            #[cfg(not(windows))]
            {
                CommandOutcome::unsupported("Core Audio is only available on Windows.")
            }
        }

        DeviceCommand::SetSystemMute { muted } => {
            #[cfg(windows)]
            {
                let actual = audio::set_mute(target_endpoint(), muted)?;
                if actual == muted {
                    CommandOutcome::applied()
                } else {
                    CommandOutcome::sent_unverified(
                        "Endpoint did not report the requested mute state.",
                    )
                }
            }
            #[cfg(not(windows))]
            {
                CommandOutcome::unsupported("Core Audio is only available on Windows.")
            }
        }

        // Media transport goes through the Windows session model, which is a
        // separate integration and not implemented yet.
        DeviceCommand::MediaPlayPause
        | DeviceCommand::MediaNext
        | DeviceCommand::MediaPrevious => {
            CommandOutcome::unsupported("Media transport control is not implemented yet.")
        }
    };

    Ok(outcome)
}

#[tauri::command]
fn get_bluetooth_availability() -> bluetooth::BluetoothAvailability {
    #[cfg(windows)]
    {
        bluetooth::radio::availability()
    }
    #[cfg(not(windows))]
    {
        bluetooth::BluetoothAvailability::unavailable("Not running on Windows.")
    }
}

/// Read-only enumeration of Bluetooth devices Windows knows about.
#[tauri::command]
fn list_bluetooth_devices() -> Vec<bluetooth::DiscoveredDevice> {
    #[cfg(windows)]
    {
        use bluetooth::{DiscoveredDevice, DiscoveredTransport};
        bluetooth::pnp::enumerate_bluetooth_devices()
            .into_iter()
            .filter(|d| d.is_top_level())
            .map(|d| {
                let name = d
                    .friendly_name
                    .clone()
                    .unwrap_or_else(|| "Unnamed device".to_string());
                DiscoveredDevice {
                    id: util::stable_id(&d.instance_id),
                    looks_like_bose: bose::looks_like_bose(&name),
                    name,
                    transport: if d.instance_id.starts_with("BTHLE") {
                        DiscoveredTransport::LowEnergy
                    } else {
                        DiscoveredTransport::Classic
                    },
                    connected: d.is_connected,
                    battery_percent: d.battery_percent,
                }
            })
            .collect()
    }
    #[cfg(not(windows))]
    {
        Vec::new()
    }
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    util::set_salt(util::generate_salt());

    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_notification::init())
        // Start on the mock backend. Switching to real hardware is an explicit
        // user action, so a fresh install never silently talks to a device.
        .manage(AppState::new(DeviceSource::Mock))
        .invoke_handler(tauri::generate_handler![
            get_snapshot,
            get_device_source,
            set_device_source,
            execute_command,
            get_bluetooth_availability,
            list_bluetooth_devices,
            list_audio_endpoints,
        ])
        .run(tauri::generate_context!())
        .expect("error while running Bose QC Control Center");
}
