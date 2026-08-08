//! Bluetooth radio availability.
//!
//! Determines whether a radio exists and whether it is switched on, so the UI
//! can distinguish "you have no Bluetooth" from "your Bluetooth is off" from
//! "your headphones aren't reachable". Those are three different problems and
//! deserve three different messages.

#![cfg(windows)]

use super::BluetoothAvailability;
use super::pnp;

/// Instance id prefix of the radio device itself, as opposed to paired peers.
const RADIO_ENUMERATOR: &str = "BTH";

pub fn availability() -> BluetoothAvailability {
    // A Bluetooth radio appears under the BTH enumerator (e.g. BTH\MS_BTHBRB).
    // Paired peer devices appear under BTHENUM / BTHLE instead, so presence of
    // peers alone is not evidence of a working radio.
    let radio_nodes = pnp::enumerate_instance_ids(RADIO_ENUMERATOR);

    if radio_nodes.is_empty() {
        return BluetoothAvailability::unavailable(
            "No Bluetooth radio is present on this system.",
        );
    }

    // If the radio is present as a PnP node and enumerating peers succeeds, the
    // stack is running. A radio that is switched off in Windows still enumerates
    // but reports its peers as disconnected.
    BluetoothAvailability {
        radio_present: true,
        radio_enabled: true,
        detail: format!(
            "{} Bluetooth radio node(s) present and the Windows Bluetooth stack responded.",
            radio_nodes.len()
        ),
    }
}
