//! Bose QC Control Center — Bluetooth discovery tool.
//!
//! **Read-only.** This tool sends nothing to any device. It reads what Windows
//! already knows about paired Bluetooth devices: names, connection state,
//! battery, and the GATT services Windows discovered during pairing. Then it
//! writes a report.
//!
//! Run it with the headphones paired and powered on:
//!
//! ```text
//! cargo run --manifest-path tools/bose-discovery/Cargo.toml
//! ```
//!
//! Reports are written to the current directory as `device-report.json` and
//! `device-report.txt`. Both contain salted hashes rather than Bluetooth
//! addresses, so they are safe to share.

// The PnP layer is shared with the application rather than duplicated. It has
// no dependencies on the rest of that crate, so including the source directly
// keeps a single source of truth without pulling Tauri into a CLI tool.
#[cfg(windows)]
#[path = "../../../app/src-tauri/src/bluetooth/pnp.rs"]
mod pnp;

mod gatt;
mod report;

use report::{DeviceRecord, Report};

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "--help" || a == "-h") {
        print_help();
        return;
    }

    println!("Bose QC Control Center — Bluetooth discovery");
    println!("Read-only. Nothing is written to any device.\n");

    #[cfg(not(windows))]
    {
        eprintln!("This tool requires Windows.");
        std::process::exit(1);
    }

    #[cfg(windows)]
    {
        let redact = args.iter().any(|a| a == "--redact-names");
        let report = collect(redact);
        print_summary(&report);

        if !redact {
            println!(
                "\nNote: device names are included as Windows reports them. Check them\n\
                 before sharing this report, or re-run with --redact-names."
            );
        }

        match report.write_to_disk(std::path::Path::new(".")) {
            Ok((json, txt)) => {
                println!("\nWrote {}", json.display());
                println!("Wrote {}", txt.display());
            }
            Err(e) => {
                eprintln!("\nFailed to write report: {e}");
                std::process::exit(1);
            }
        }
    }
}

fn print_help() {
    println!(
        "bose-discovery — read-only Bluetooth discovery\n\n\
         Reads paired-device information Windows already holds and writes\n\
         device-report.json and device-report.txt to the current directory.\n\n\
         This tool never writes to a Bluetooth device.\n\n\
         Options:\n  \
         -h, --help        Show this help\n  \
         --redact-names    Replace non-Bose device names with a placeholder,\n                    \
         for reports you intend to share"
    );
}

#[cfg(windows)]
fn collect(redact: bool) -> Report {
    use std::collections::BTreeMap;

    let salt = report::session_salt();

    // Group GATT service nodes by the device address they belong to, so each
    // device's service list can be attached to it.
    let mut services_by_address: BTreeMap<String, Vec<gatt::GattService>> = BTreeMap::new();
    for instance_id in pnp::enumerate_instance_ids("BTHLEDEVICE") {
        if let (Some(service), Some(address)) = (
            gatt::parse_service(&instance_id),
            gatt::owning_address(&instance_id),
        ) {
            let entry = services_by_address.entry(address).or_default();
            if !entry.iter().any(|s| s.uuid == service.uuid) {
                entry.push(service);
            }
        }
    }

    let radio_nodes = pnp::enumerate_instance_ids("BTH");

    let mut devices = Vec::new();
    for dev in pnp::enumerate_bluetooth_devices() {
        if !dev.is_top_level() {
            continue;
        }
        let name = dev
            .friendly_name
            .clone()
            .unwrap_or_else(|| "Unnamed device".to_string());

        let address = extract_address(&dev.instance_id);
        let services = address
            .as_ref()
            .and_then(|a| services_by_address.get(a).cloned())
            .unwrap_or_default();

        let is_bose = looks_like_bose(&name);
        devices.push(DeviceRecord {
            id: report::stable_id(&salt, &dev.instance_id),
            name: if redact {
                report::redact_name(&name, is_bose)
            } else {
                name.clone()
            },
            transport: if dev.instance_id.starts_with("BTHLE") {
                "bluetooth-le".to_string()
            } else {
                "bluetooth-classic".to_string()
            },
            connected: dev.is_connected,
            battery_percent: dev.battery_percent,
            looks_like_bose: is_bose,
            gatt_services: services,
        });
    }

    Report::new(radio_nodes.len(), devices)
}

/// Pulls the 12-hex-digit address out of a top-level instance id.
#[cfg(windows)]
fn extract_address(instance_id: &str) -> Option<String> {
    let upper = instance_id.to_uppercase();
    let after_dev = upper.split("DEV_").nth(1)?;
    let candidate: String = after_dev.chars().take(12).collect();
    if candidate.len() == 12 && candidate.chars().all(|c| c.is_ascii_hexdigit()) {
        Some(candidate)
    } else {
        None
    }
}

/// Kept in sync with `app/src-tauri/src/bose/mod.rs`.
fn looks_like_bose(name: &str) -> bool {
    let lowered = name.to_lowercase();
    ["bose", "quietcomfort", "quiet comfort", "qc45", "qc35", "qc ultra"]
        .iter()
        .any(|hint| lowered.contains(hint))
}

#[cfg(windows)]
fn print_summary(report: &Report) {
    println!("Bluetooth radio nodes present: {}", report.radio_nodes);
    println!("Paired devices found: {}\n", report.devices.len());

    let bose: Vec<_> = report.devices.iter().filter(|d| d.looks_like_bose).collect();

    if bose.is_empty() {
        println!("No device matching a Bose name hint was found.");
        println!("If your headphones are paired, check that they are powered on");
        println!("and connected to this PC rather than another device.\n");
    } else {
        println!("Possible Bose device(s):");
        for d in &bose {
            println!("  {} — {}", d.name, connection_text(d));
        }
        println!();
    }

    // Transport is shown because a device paired over both Classic and LE
    // legitimately appears twice, and without it the two rows look identical.
    println!("All paired devices:");
    for d in &report.devices {
        let transport = match d.transport.as_str() {
            "bluetooth-le" => "LE",
            _ => "Classic",
        };
        println!(
            "  {:<34} [{:<7}] {}",
            truncate(&d.name, 34),
            transport,
            connection_text(d)
        );
        for s in &d.gatt_services {
            let label = s
                .known_name
                .clone()
                .unwrap_or_else(|| "vendor-specific".to_string());
            println!("      GATT {} — {}", s.uuid, label);
        }
    }
}

#[cfg(windows)]
fn connection_text(d: &DeviceRecord) -> String {
    let conn = match d.connected {
        Some(true) => "connected",
        Some(false) => "not connected",
        None => "state unknown",
    };
    match d.battery_percent {
        Some(b) => format!("{conn}, battery {b}%"),
        None => format!("{conn}, no battery reported"),
    }
}

fn truncate(s: &str, n: usize) -> String {
    if s.chars().count() <= n {
        s.to_string()
    } else {
        s.chars().take(n.saturating_sub(1)).collect::<String>() + "…"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn bose_hints_match_the_application() {
        assert!(looks_like_bose("Bose QuietComfort Headphones"));
        assert!(!looks_like_bose("Jaisal's S24 Ultra"));
    }

    #[test]
    fn truncate_preserves_short_names() {
        assert_eq!(truncate("Bose QC", 42), "Bose QC");
    }

    #[cfg(windows)]
    #[test]
    fn extracts_address_from_top_level_ids() {
        assert_eq!(
            extract_address("BTHENUM\\DEV_E458BCF9F02E\\7&78167D1&0&BLUETOOTHDEVICE_E458BCF9F02E")
                .as_deref(),
            Some("E458BCF9F02E")
        );
        assert_eq!(
            extract_address("BTHLE\\DEV_79C657FDB4BC\\7&1E36B139&0&79C657FDB4BC").as_deref(),
            Some("79C657FDB4BC")
        );
    }
}
