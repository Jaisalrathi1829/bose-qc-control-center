//! Offline btsnoop analyser.
//!
//! Reads an Android Bluetooth HCI snoop log, filters it to one device, and
//! reports the RFCOMM traffic that device exchanged. Entirely offline: it
//! reads a file and writes a report. It never touches Bluetooth hardware.
//!
//! Privacy: a snoop log contains every Bluetooth device the phone talked to.
//! This tool filters to a single address and discards everything else, so the
//! exported report cannot leak traffic from unrelated devices.

mod btsnoop;
mod stack;

use btsnoop::{format_time, Direction};
use serde::Serialize;
use stack::{Decoder, RfcommFrame, RfcommFrameKind};
use std::collections::BTreeMap;

/// Bose Corporation's Bluetooth SIG company identifier.
const BOSE_SIG_COMPANY_ID: u16 = 0x009E;

#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ExportedFrame {
    time: String,
    timestamp_us: i64,
    direction: &'static str,
    dlci: u8,
    frame_type: String,
    length: usize,
    hex: String,
    ascii: String,
}

fn main() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.is_empty() || args.iter().any(|a| a == "-h" || a == "--help") {
        print_help();
        return;
    }

    let path = &args[0];
    let filter_address = args
        .iter()
        .position(|a| a == "--address")
        .and_then(|i| args.get(i + 1))
        .map(|s| s.to_uppercase().replace(':', ""));

    let bytes = match std::fs::read(path) {
        Ok(b) => b,
        Err(e) => {
            eprintln!("Could not read {path}: {e}");
            std::process::exit(1);
        }
    };

    println!("Bose QC Control Center — btsnoop analyser");
    println!("=========================================\n");
    println!("Reading {path} ({:.1} MB)\n", bytes.len() as f64 / 1_048_576.0);

    let records = match btsnoop::parse(&bytes) {
        Ok(r) => r,
        Err(e) => {
            eprintln!("{e}");
            std::process::exit(1);
        }
    };
    println!("HCI records: {}", records.len());

    let mut decoder = Decoder::new();
    let mut frames: Vec<RfcommFrame> = Vec::new();
    for rec in &records {
        frames.extend(decoder.push(rec));
    }

    // Which devices produced RFCOMM traffic at all.
    let mut by_address: BTreeMap<String, usize> = BTreeMap::new();
    for f in &frames {
        *by_address
            .entry(f.address.clone().unwrap_or_else(|| "unknown".into()))
            .or_insert(0) += 1;
    }

    println!("RFCOMM frames: {}\n", frames.len());
    if frames.is_empty() {
        println!("No RFCOMM traffic was found in this log.\n");
        println!("Possible reasons:");
        println!("  - The snoop log was started after the app had already connected,");
        println!("    so the L2CAP handshake that identifies the RFCOMM channel was");
        println!("    missed. Toggle Bluetooth off and on with logging enabled, then");
        println!("    reconnect and reproduce the actions.");
        println!("  - The app used BLE GATT rather than RFCOMM.");
        return;
    }

    println!("Devices with RFCOMM traffic:");
    for (addr, count) in &by_address {
        println!("  {addr}  {count} frames");
    }
    println!();

    let target = match &filter_address {
        Some(a) => Some(a.clone()),
        None => {
            // Default to whichever device produced the most RFCOMM traffic.
            by_address
                .iter()
                .filter(|(a, _)| a.as_str() != "unknown")
                .max_by_key(|(_, c)| **c)
                .map(|(a, _)| a.clone())
        }
    };

    let Some(target) = target else {
        eprintln!("No device address could be determined. Pass --address explicitly.");
        std::process::exit(1);
    };

    println!("Filtering to {target} — all other devices discarded.\n");

    let selected: Vec<&RfcommFrame> = frames
        .iter()
        .filter(|f| f.address.as_deref() == Some(target.as_str()))
        .collect();

    // Payload-bearing frames are the interesting ones; the rest is link setup.
    let data_frames: Vec<&&RfcommFrame> = selected
        .iter()
        .filter(|f| f.kind == RfcommFrameKind::Uih && !f.payload.is_empty())
        .collect();

    println!("Frames for this device : {}", selected.len());
    println!("With a payload         : {}\n", data_frames.len());

    report(&selected, &data_frames, &target);
}

fn to_hex(b: &[u8]) -> String {
    b.iter().map(|x| format!("{x:02X}")).collect::<Vec<_>>().join(" ")
}

fn to_ascii(b: &[u8]) -> String {
    b.iter()
        .map(|&c| if (0x20..0x7f).contains(&c) { c as char } else { '.' })
        .collect()
}

fn report(all: &[&RfcommFrame], data: &[&&RfcommFrame], address: &str) {
    // --- JSONL of payload frames ---
    let exported: Vec<ExportedFrame> = data
        .iter()
        .map(|f| ExportedFrame {
            time: format_time(f.timestamp_us),
            timestamp_us: f.timestamp_us,
            direction: match f.direction {
                Direction::Sent => "PHONE->DEVICE",
                Direction::Received => "DEVICE->PHONE",
            },
            dlci: f.dlci,
            frame_type: f.kind.label(),
            length: f.payload.len(),
            hex: to_hex(&f.payload),
            ascii: to_ascii(&f.payload),
        })
        .collect();

    let jsonl = exported
        .iter()
        .filter_map(|e| serde_json::to_string(e).ok())
        .collect::<Vec<_>>()
        .join("\n");
    let _ = std::fs::write("btsnoop-rfcomm.jsonl", jsonl);

    // --- text report ---
    let mut t = String::new();
    t.push_str("BOSE btsnoop RFCOMM EXTRACT\n===========================\n\n");
    t.push_str(&format!("Device        : {address}\n"));
    t.push_str(&format!("RFCOMM frames : {}\n", all.len()));
    t.push_str(&format!("With payload  : {}\n\n", data.len()));
    t.push_str(
        "Only traffic for the device above is included. Every other device in\n\
         the source log was discarded.\n\n",
    );

    // Channel setup, useful for spotting which DLCI carries what.
    t.push_str("LINK EVENTS\n-----------\n");
    for f in all.iter().filter(|f| f.kind != RfcommFrameKind::Uih) {
        t.push_str(&format!(
            "  {}  {}  DLCI {:>2}  {}\n",
            format_time(f.timestamp_us),
            f.direction.label(),
            f.dlci,
            f.kind.label()
        ));
    }

    t.push_str("\nPAYLOAD FRAMES\n--------------\n");
    for f in data {
        t.push_str(&format!(
            "  {}  {}  DLCI {:>2}  {:>3}B  {}  |{}|\n",
            format_time(f.timestamp_us),
            f.direction.label(),
            f.dlci,
            f.payload.len(),
            to_hex(&f.payload),
            to_ascii(&f.payload)
        ));
    }

    // --- grouping, without interpretation ---
    let mut by_dlci: BTreeMap<u8, usize> = BTreeMap::new();
    let mut by_prefix: BTreeMap<String, usize> = BTreeMap::new();
    for f in data {
        *by_dlci.entry(f.dlci).or_insert(0) += 1;
        let prefix: Vec<u8> = f.payload.iter().take(4).copied().collect();
        *by_prefix.entry(to_hex(&prefix)).or_insert(0) += 1;
    }

    t.push_str("\nPAYLOAD FRAMES PER DLCI\n-----------------------\n");
    for (dlci, n) in &by_dlci {
        t.push_str(&format!("  DLCI {dlci:>2} : {n} frames\n"));
    }

    t.push_str("\nMOST COMMON 4-BYTE PREFIXES\n---------------------------\n");
    let mut prefixes: Vec<_> = by_prefix.into_iter().collect();
    prefixes.sort_by(|a, b| b.1.cmp(&a.1));
    for (prefix, n) in prefixes.iter().take(25) {
        t.push_str(&format!("  {prefix}  x{n}\n"));
    }

    t.push_str(
        "\nINTERPRETATION\n--------------\n\
         None. This report groups and counts observed bytes and nothing more.\n\n\
         A recurring prefix is not necessarily an opcode, a differing byte is\n\
         not necessarily a parameter, and a frame sent near an action is not\n\
         proof that it encodes that action. Assigning meaning requires\n\
         correlating several captures of deliberately varied single actions.\n\n\
         Capability statuses must NOT be changed on the strength of this file\n\
         alone, and nothing here authorises transmitting anything to the\n\
         device.\n",
    );

    let _ = std::fs::write("btsnoop-rfcomm.txt", &t);

    // --- console summary ---
    println!("Payload frames per DLCI:");
    for (dlci, n) in &by_dlci {
        println!("  DLCI {dlci:>2} : {n}");
    }

    println!("\nFirst payload frames:");
    for f in data.iter().take(12) {
        println!(
            "  {}  {}  DLCI {:>2}  {:>3}B  {}",
            format_time(f.timestamp_us),
            f.direction.label(),
            f.dlci,
            f.payload.len(),
            truncate(&to_hex(&f.payload), 48)
        );
    }

    println!("\nWrote btsnoop-rfcomm.jsonl");
    println!("Wrote btsnoop-rfcomm.txt");
}

fn truncate(s: &str, n: usize) -> String {
    if s.len() <= n {
        s.to_string()
    } else {
        format!("{}…", &s[..n])
    }
}

fn print_help() {
    println!(
        "bose-btsnoop-parse — extract RFCOMM traffic from an Android HCI snoop log\n\n\
         Offline. Reads a file and writes a report; never touches Bluetooth hardware.\n\n\
         Usage:\n  \
         bose-btsnoop-parse <btsnoop_hci.log> [--address E458BCF9F02E]\n\n\
         Options:\n  \
         --address <addr>   Only include this device. Defaults to whichever\n                     \
         device produced the most RFCOMM traffic.\n  \
         -h, --help         Show this help\n\n\
         Writes btsnoop-rfcomm.jsonl and btsnoop-rfcomm.txt.\n\n\
         Note: Bose's SIG company id is 0x{BOSE_SIG_COMPANY_ID:04X}; the headphones on the\n\
         development machine are at address E458BCF9F02E."
    );
}
