//! Windows PnP (Configuration Manager) access.
//!
//! Why CfgMgr32 rather than WinRT here: this path is synchronous, needs no COM
//! apartment, and gives direct access to device properties — including the
//! Bluetooth battery property that Windows itself uses to show headphone
//! battery in Settings. WinRT's `DeviceInformation` exposes the same data but
//! requires constructing an `IIterable<HSTRING>` of property keys, which is
//! awkward from Rust for no benefit at this layer.
//!
//! Everything in this module is read-only. Nothing here writes to a device.

#![cfg(windows)]

use std::collections::BTreeMap;
use windows::core::PCWSTR;
use windows::Win32::Devices::DeviceAndDriverInstallation::{
    CM_Get_DevNode_PropertyW, CM_Get_Device_ID_ListW, CM_Get_Device_ID_List_SizeW,
    CM_Locate_DevNodeW, CM_LOCATE_DEVNODE_NORMAL, CM_GETIDLIST_FILTER_PRESENT, CR_SUCCESS,
};
use windows::Win32::Devices::Properties::{
    DEVPROPKEY, DEVPROPTYPE, DEVPROP_TYPE_BOOLEAN, DEVPROP_TYPE_BYTE, DEVPROP_TYPE_STRING,
    DEVPROP_TYPE_UINT32,
};

/// `DEVPKEY_Device_FriendlyName` — {a45c254e-df1c-4efd-8020-67d146a850e0}, 14
pub const DEVPKEY_DEVICE_FRIENDLY_NAME: DEVPROPKEY = DEVPROPKEY {
    fmtid: windows::core::GUID::from_u128(0xa45c254e_df1c_4efd_8020_67d146a850e0),
    pid: 14,
};

/// `DEVPKEY_Device_DeviceDesc` — {a45c254e-df1c-4efd-8020-67d146a850e0}, 2
pub const DEVPKEY_DEVICE_DESC: DEVPROPKEY = DEVPROPKEY {
    fmtid: windows::core::GUID::from_u128(0xa45c254e_df1c_4efd_8020_67d146a850e0),
    pid: 2,
};

/// Bluetooth device battery level, 0-100.
///
/// {104EA319-6EE2-4701-BD47-8DDBF425BBE5}, 2
///
/// This is the property Windows Settings uses to display headphone battery.
/// It is populated by the HFP battery indication (`AT+IPHONEACCEV` / Android's
/// battery AT command) or by the BLE Battery Service, depending on the device.
/// Whether any given device populates it must be checked per device — the
/// mechanism working is not evidence that a specific device supports it.
pub const DEVPKEY_BLUETOOTH_BATTERY: DEVPROPKEY = DEVPROPKEY {
    fmtid: windows::core::GUID::from_u128(0x104ea319_6ee2_4701_bd47_8ddbf425bbe5),
    pid: 2,
};

/// Device connection state. {83DA6326-97A6-4088-9453-A1923F573B29}, 15
pub const DEVPKEY_DEVICE_IS_CONNECTED: DEVPROPKEY = DEVPROPKEY {
    fmtid: windows::core::GUID::from_u128(0x83da6326_97a6_4088_9453_a1923f573b29),
    pid: 15,
};

/// A value read from a PnP property.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PropValue {
    Text(String),
    Bool(bool),
    Number(u32),
    Byte(u8),
}

impl PropValue {
    pub fn as_text(&self) -> Option<&str> {
        match self {
            Self::Text(s) => Some(s.as_str()),
            _ => None,
        }
    }

    pub fn as_bool(&self) -> Option<bool> {
        match self {
            Self::Bool(b) => Some(*b),
            _ => None,
        }
    }

    /// Battery is reported as a byte on most stacks but a UINT32 on some.
    pub fn as_percent(&self) -> Option<u8> {
        match self {
            Self::Byte(b) => Some(*b),
            Self::Number(n) if *n <= 100 => Some(*n as u8),
            _ => None,
        }
    }
}

/// A Bluetooth-related device node as Windows sees it.
#[derive(Debug, Clone)]
pub struct PnpDevice {
    pub instance_id: String,
    pub friendly_name: Option<String>,
    pub is_connected: Option<bool>,
    pub battery_percent: Option<u8>,
}

impl PnpDevice {
    /// Whether this instance id refers to a top-level device rather than one of
    /// the per-profile child nodes Windows creates (AVRCP transport, Handsfree
    /// service, and so on). Those children duplicate the friendly name and
    /// would otherwise produce many spurious "devices".
    pub fn is_top_level(&self) -> bool {
        let id = self.instance_id.to_uppercase();
        // Child profile nodes carry a service GUID in braces.
        let has_service_guid = id.contains('{');
        (id.starts_with("BTHENUM\\DEV_") || id.starts_with("BTHLE\\DEV_")) && !has_service_guid
    }
}

fn wide_to_string(buf: &[u16]) -> String {
    let end = buf.iter().position(|&c| c == 0).unwrap_or(buf.len());
    String::from_utf16_lossy(&buf[..end])
}

fn to_wide(s: &str) -> Vec<u16> {
    s.encode_utf16().chain(std::iter::once(0)).collect()
}

/// Reads a single property from a device node.
///
/// Returns `None` when the property is simply not present, which is the common
/// case and not an error — most devices do not report battery.
pub fn get_property(instance_id: &str, key: &DEVPROPKEY) -> Option<PropValue> {
    unsafe {
        let wide = to_wide(instance_id);
        let mut devinst = 0u32;
        let cr = CM_Locate_DevNodeW(
            &mut devinst,
            PCWSTR(wide.as_ptr()),
            CM_LOCATE_DEVNODE_NORMAL,
        );
        if cr != CR_SUCCESS {
            return None;
        }

        let mut prop_type = DEVPROPTYPE(0);
        let mut size = 0u32;

        // First call sizes the buffer.
        let _ = CM_Get_DevNode_PropertyW(devinst, key, &mut prop_type, None, &mut size, 0);
        if size == 0 {
            return None;
        }

        let mut buf = vec![0u8; size as usize];
        let cr = CM_Get_DevNode_PropertyW(
            devinst,
            key,
            &mut prop_type,
            Some(buf.as_mut_ptr()),
            &mut size,
            0,
        );
        if cr != CR_SUCCESS {
            return None;
        }
        buf.truncate(size as usize);

        match prop_type.0 {
            t if t == DEVPROP_TYPE_STRING.0 => {
                let u16buf: Vec<u16> = buf
                    .chunks_exact(2)
                    .map(|c| u16::from_le_bytes([c[0], c[1]]))
                    .collect();
                Some(PropValue::Text(wide_to_string(&u16buf)))
            }
            t if t == DEVPROP_TYPE_BOOLEAN.0 => buf.first().map(|b| PropValue::Bool(*b != 0)),
            t if t == DEVPROP_TYPE_BYTE.0 => buf.first().map(|b| PropValue::Byte(*b)),
            t if t == DEVPROP_TYPE_UINT32.0 => {
                if buf.len() >= 4 {
                    Some(PropValue::Number(u32::from_le_bytes([
                        buf[0], buf[1], buf[2], buf[3],
                    ])))
                } else {
                    None
                }
            }
            _ => None,
        }
    }
}

/// Enumerates present device instance ids under the given enumerator prefix.
///
/// `filter` is a PnP enumerator such as `"BTHENUM"` (Bluetooth Classic) or
/// `"BTHLE"` (Bluetooth LE). Only devices currently present are returned.
pub fn enumerate_instance_ids(filter: &str) -> Vec<String> {
    unsafe {
        let wide = to_wide(filter);
        let mut len = 0u32;
        let cr = CM_Get_Device_ID_List_SizeW(
            &mut len,
            PCWSTR(wide.as_ptr()),
            CM_GETIDLIST_FILTER_PRESENT | CM_GETIDLIST_FILTER_ENUMERATOR,
        );
        if cr != CR_SUCCESS || len == 0 {
            return Vec::new();
        }

        let mut buf = vec![0u16; len as usize];
        let cr = CM_Get_Device_ID_ListW(
            PCWSTR(wide.as_ptr()),
            &mut buf,
            CM_GETIDLIST_FILTER_PRESENT | CM_GETIDLIST_FILTER_ENUMERATOR,
        );
        if cr != CR_SUCCESS {
            return Vec::new();
        }

        // The result is a double-null-terminated multi-string.
        buf.split(|&c| c == 0)
            .filter(|s| !s.is_empty())
            .map(|s| String::from_utf16_lossy(s))
            .collect()
    }
}

const CM_GETIDLIST_FILTER_ENUMERATOR: u32 = 0x00000001;

/// Enumerates Bluetooth devices (Classic and LE) with their readable properties.
pub fn enumerate_bluetooth_devices() -> Vec<PnpDevice> {
    let mut out = Vec::new();
    let mut seen = BTreeMap::new();

    for enumerator in ["BTHENUM", "BTHLE"] {
        for instance_id in enumerate_instance_ids(enumerator) {
            let friendly_name = get_property(&instance_id, &DEVPKEY_DEVICE_FRIENDLY_NAME)
                .and_then(|v| v.as_text().map(str::to_string))
                .or_else(|| {
                    get_property(&instance_id, &DEVPKEY_DEVICE_DESC)
                        .and_then(|v| v.as_text().map(str::to_string))
                });

            let is_connected =
                get_property(&instance_id, &DEVPKEY_DEVICE_IS_CONNECTED).and_then(|v| v.as_bool());

            let battery_percent =
                get_property(&instance_id, &DEVPKEY_BLUETOOTH_BATTERY).and_then(|v| v.as_percent());

            let dev = PnpDevice {
                instance_id: instance_id.clone(),
                friendly_name,
                is_connected,
                battery_percent,
            };
            seen.insert(instance_id, dev);
        }
    }

    out.extend(seen.into_values());
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn dev(instance_id: &str) -> PnpDevice {
        PnpDevice {
            instance_id: instance_id.to_string(),
            friendly_name: None,
            is_connected: None,
            battery_percent: None,
        }
    }

    #[test]
    fn top_level_classic_and_le_nodes_are_recognised() {
        assert!(dev("BTHENUM\\DEV_E458BCF9F02E\\7&78167D1&0&BLUETOOTHDEVICE_E458BCF9F02E")
            .is_top_level());
        assert!(dev("BTHLE\\DEV_79C657FDB4BC\\7&1E36B139&0&79C657FDB4BC").is_top_level());
    }

    /// Per-profile child nodes must be filtered out, otherwise a single pair of
    /// headphones appears as half a dozen separate devices.
    #[test]
    fn per_profile_child_nodes_are_excluded() {
        assert!(!dev(
            "BTHENUM\\{0000110C-0000-1000-8000-00805F9B34FB}_VID&00010075_PID&0100\\7&78167D1&0&34F043C9E0F6_C00000000"
        )
        .is_top_level());
        assert!(!dev(
            "BTHLEDEVICE\\{0000180F-0000-1000-8000-00805F9B34FB}_DEV_VID&0017EF_PID&6134_REV&0026_79C657FDB4BC\\8&30736FC1&0&0020"
        )
        .is_top_level());
    }

    #[test]
    fn percent_accepts_byte_and_bounded_u32() {
        assert_eq!(PropValue::Byte(60).as_percent(), Some(60));
        assert_eq!(PropValue::Number(60).as_percent(), Some(60));
        // A UINT32 outside 0-100 is not a percentage; refuse rather than clamp.
        assert_eq!(PropValue::Number(4294967295).as_percent(), None);
        assert_eq!(PropValue::Text("60".into()).as_percent(), None);
    }
}
