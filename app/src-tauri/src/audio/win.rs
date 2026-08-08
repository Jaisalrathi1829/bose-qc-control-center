//! Core Audio implementation.
//!
//! All COM. Each public function initialises COM for the calling thread
//! (idempotent — `CoInitializeEx` returns `S_FALSE` if already initialised on
//! that thread) because Tauri commands run on a pool and we cannot assume which
//! thread we land on.

#![cfg(windows)]

use super::{looks_like_bluetooth_endpoint, percent_to_scalar, scalar_to_percent, AudioEndpoint};
use crate::device::{DeviceError, DeviceResult};
use windows::core::PCWSTR;
use windows::Win32::Devices::FunctionDiscovery::PKEY_Device_FriendlyName;
use windows::Win32::Media::Audio::Endpoints::IAudioEndpointVolume;
use windows::Win32::Media::Audio::{
    eCommunications, eConsole, eRender, IAudioClient, IMMDevice, IMMDeviceEnumerator,
    MMDeviceEnumerator, DEVICE_STATE_ACTIVE,
};
use windows::Win32::System::Com::{
    CoCreateInstance, CoInitializeEx, CoTaskMemFree, CLSCTX_ALL, COINIT_MULTITHREADED, STGM_READ,
};

fn com_error(e: windows::core::Error) -> DeviceError {
    DeviceError::Platform(format!("Core Audio: {e}"))
}

/// Initialises COM for this thread. Safe to call repeatedly.
fn ensure_com() {
    unsafe {
        // Returns S_FALSE when already initialised on this thread, which is
        // not an error for our purposes.
        let _ = CoInitializeEx(None, COINIT_MULTITHREADED);
    }
}

fn enumerator() -> DeviceResult<IMMDeviceEnumerator> {
    ensure_com();
    unsafe {
        CoCreateInstance::<_, IMMDeviceEnumerator>(&MMDeviceEnumerator, None, CLSCTX_ALL)
            .map_err(com_error)
    }
}

/// Reads an endpoint's friendly name from its property store.
///
/// `PROPVARIANT` owns its buffer and implements `Display`, so there is nothing
/// to free by hand here.
fn endpoint_name(device: &IMMDevice) -> DeviceResult<String> {
    unsafe {
        let store = device.OpenPropertyStore(STGM_READ).map_err(com_error)?;
        let variant = store.GetValue(&PKEY_Device_FriendlyName).map_err(com_error)?;
        Ok(variant.to_string())
    }
}

fn endpoint_id(device: &IMMDevice) -> DeviceResult<String> {
    unsafe {
        let pwstr = device.GetId().map_err(com_error)?;
        let id = pwstr.to_string().unwrap_or_default();
        CoTaskMemFree(Some(pwstr.0 as *const _));
        Ok(id)
    }
}

/// Mix format, when the endpoint exposes one. Absent is normal, not an error.
fn mix_format(device: &IMMDevice) -> Option<(u32, u16)> {
    unsafe {
        let client = device.Activate::<IAudioClient>(CLSCTX_ALL, None).ok()?;
        let fmt = client.GetMixFormat().ok()?;
        if fmt.is_null() {
            return None;
        }
        let rate = (*fmt).nSamplesPerSec;
        let channels = (*fmt).nChannels;
        CoTaskMemFree(Some(fmt as *const _));
        Some((rate, channels))
    }
}

fn volume_interface(device: &IMMDevice) -> DeviceResult<IAudioEndpointVolume> {
    unsafe {
        device
            .Activate::<IAudioEndpointVolume>(CLSCTX_ALL, None)
            .map_err(com_error)
    }
}

fn build_endpoint(
    device: &IMMDevice,
    default_render_id: Option<&str>,
    default_comms_id: Option<&str>,
) -> DeviceResult<AudioEndpoint> {
    let id = endpoint_id(device)?;
    let name = endpoint_name(device).unwrap_or_else(|_| "Unknown endpoint".to_string());

    let vol = volume_interface(device)?;
    let (volume_percent, muted) = unsafe {
        let scalar = vol.GetMasterVolumeLevelScalar().map_err(com_error)?;
        let muted = vol.GetMute().map_err(com_error)?.as_bool();
        (scalar_to_percent(scalar), muted)
    };

    let (sample_rate_hz, channels) = match mix_format(device) {
        Some((r, c)) => (Some(r), Some(c)),
        None => (None, None),
    };

    Ok(AudioEndpoint {
        is_default_render: default_render_id == Some(id.as_str()),
        is_default_communications: default_comms_id == Some(id.as_str()),
        likely_bluetooth: looks_like_bluetooth_endpoint(&name),
        id,
        name,
        volume_percent,
        muted,
        sample_rate_hz,
        channels,
    })
}

/// Enumerates active audio output endpoints.
pub fn list_output_endpoints() -> DeviceResult<Vec<AudioEndpoint>> {
    let enumerator = enumerator()?;

    let default_render = unsafe {
        enumerator
            .GetDefaultAudioEndpoint(eRender, eConsole)
            .ok()
            .and_then(|d| endpoint_id(&d).ok())
    };
    let default_comms = unsafe {
        enumerator
            .GetDefaultAudioEndpoint(eRender, eCommunications)
            .ok()
            .and_then(|d| endpoint_id(&d).ok())
    };

    unsafe {
        let collection = enumerator
            .EnumAudioEndpoints(eRender, DEVICE_STATE_ACTIVE)
            .map_err(com_error)?;
        let count = collection.GetCount().map_err(com_error)?;

        let mut out = Vec::with_capacity(count as usize);
        for i in 0..count {
            let Ok(device) = collection.Item(i) else {
                continue;
            };
            // A single failing endpoint must not fail the whole enumeration.
            if let Ok(ep) = build_endpoint(&device, default_render.as_deref(), default_comms.as_deref())
            {
                out.push(ep);
            }
        }
        Ok(out)
    }
}

/// The current default output endpoint, if any.
pub fn default_output_endpoint() -> DeviceResult<Option<AudioEndpoint>> {
    let enumerator = enumerator()?;
    unsafe {
        let Ok(device) = enumerator.GetDefaultAudioEndpoint(eRender, eConsole) else {
            // No active output device at all is a normal state.
            return Ok(None);
        };
        let default_render = endpoint_id(&device).ok();
        let default_comms = enumerator
            .GetDefaultAudioEndpoint(eRender, eCommunications)
            .ok()
            .and_then(|d| endpoint_id(&d).ok());

        Ok(Some(build_endpoint(
            &device,
            default_render.as_deref(),
            default_comms.as_deref(),
        )?))
    }
}

fn device_by_id(id: &str) -> DeviceResult<IMMDevice> {
    let enumerator = enumerator()?;
    let wide: Vec<u16> = id.encode_utf16().chain(std::iter::once(0)).collect();
    unsafe {
        enumerator
            .GetDevice(PCWSTR(wide.as_ptr()))
            .map_err(com_error)
    }
}

/// Sets the system volume for an endpoint, then re-reads to confirm.
///
/// Returns the value the endpoint actually reports afterwards, which is what
/// the caller should believe — not the value that was requested.
pub fn set_volume(endpoint_id: Option<&str>, percent: u8) -> DeviceResult<u8> {
    if percent > 100 {
        return Err(DeviceError::InvalidInput(format!(
            "volume must be 0-100, got {percent}"
        )));
    }

    let device = match endpoint_id {
        Some(id) => device_by_id(id)?,
        None => {
            let enumerator = enumerator()?;
            unsafe {
                enumerator
                    .GetDefaultAudioEndpoint(eRender, eConsole)
                    .map_err(com_error)?
            }
        }
    };

    let vol = volume_interface(&device)?;
    unsafe {
        vol.SetMasterVolumeLevelScalar(percent_to_scalar(percent), std::ptr::null())
            .map_err(com_error)?;
        let actual = vol.GetMasterVolumeLevelScalar().map_err(com_error)?;
        Ok(scalar_to_percent(actual))
    }
}

/// Sets mute for an endpoint, then re-reads to confirm.
pub fn set_mute(endpoint_id: Option<&str>, muted: bool) -> DeviceResult<bool> {
    let device = match endpoint_id {
        Some(id) => device_by_id(id)?,
        None => {
            let enumerator = enumerator()?;
            unsafe {
                enumerator
                    .GetDefaultAudioEndpoint(eRender, eConsole)
                    .map_err(com_error)?
            }
        }
    };

    let vol = volume_interface(&device)?;
    unsafe {
        vol.SetMute(muted, std::ptr::null()).map_err(com_error)?;
        Ok(vol.GetMute().map_err(com_error)?.as_bool())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Exercises the real COM path against whatever audio hardware the machine
    /// has. Ignored by default because it depends on the environment — a CI box
    /// with no audio endpoint would fail it for the wrong reason.
    ///
    /// Run explicitly with:
    /// `cargo test --manifest-path app/src-tauri/Cargo.toml -- --ignored --nocapture`
    #[test]
    #[ignore = "requires real audio hardware"]
    fn enumerates_real_endpoints() {
        let endpoints = list_output_endpoints().expect("Core Audio enumeration failed");
        println!("Found {} output endpoint(s):", endpoints.len());
        for ep in &endpoints {
            println!(
                "  {:<45} vol={:>3}% muted={} default={} bt={} {}",
                ep.name,
                ep.volume_percent,
                ep.muted,
                ep.is_default_render,
                ep.likely_bluetooth,
                ep.sample_rate_hz
                    .map(|r| format!("{r}Hz"))
                    .unwrap_or_default()
            );
            assert!(ep.volume_percent <= 100, "volume out of range");
            assert!(!ep.id.is_empty(), "endpoint id must not be empty");
        }
        assert!(
            !endpoints.is_empty(),
            "expected at least one active output endpoint"
        );
    }

    #[test]
    #[ignore = "requires real audio hardware"]
    fn reads_default_endpoint() {
        let ep = default_output_endpoint()
            .expect("Core Audio call failed")
            .expect("no default output endpoint");
        println!("Default endpoint: {} at {}%", ep.name, ep.volume_percent);
        assert!(ep.is_default_render);
    }

    /// Sets the volume to whatever it already is, then confirms the read-back.
    /// Deliberately a no-op change so running the suite cannot disturb the
    /// machine's actual audio settings.
    #[test]
    #[ignore = "requires real audio hardware"]
    fn volume_round_trip_is_non_destructive() {
        let before = default_output_endpoint().unwrap().expect("no endpoint");
        let actual = set_volume(None, before.volume_percent).expect("set_volume failed");
        assert_eq!(
            actual, before.volume_percent,
            "endpoint did not report back the value we set"
        );
    }
}
