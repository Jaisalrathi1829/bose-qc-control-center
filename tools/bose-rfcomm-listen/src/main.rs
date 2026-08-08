//! Listen-only RFCOMM capture for a Bose device.
//!
//! Opens the vendor-specific RFCOMM service, transmits **nothing**, and logs
//! every byte the device volunteers. The operator marks physical actions as
//! they perform them so frames can be correlated against them afterwards.
//!
//! See `rfcomm.rs` for how the no-write guarantee is enforced.

#[cfg(windows)]
#[path = "../../../app/src-tauri/src/bluetooth/pnp.rs"]
mod pnp;

mod capture;
#[cfg(windows)]
mod rfcomm;

#[cfg(windows)]
use capture::{Action, Event, KEY_HELP};
#[cfg(windows)]
use std::io::{BufRead, Write};
#[cfg(windows)]
use std::sync::atomic::{AtomicBool, Ordering};
#[cfg(windows)]
use std::sync::{Arc, Mutex};
#[cfg(windows)]
use std::time::{Duration, Instant};

/// The vendor-specific RFCOMM service observed on the test headphones.
#[cfg(windows)]
const VENDOR_RFCOMM_UUID: windows::core::GUID =
    windows::core::GUID::from_u128(0x9B26D8C0_A8ED_440B_95B0_C4714A518BCC);
#[cfg(windows)]
const VENDOR_RFCOMM_UUID_TEXT: &str = "{9B26D8C0-A8ED-440B-95B0-C4714A518BCC}";

/// Bose Corporation's Bluetooth SIG company identifier.
#[cfg(windows)]
const BOSE_SIG_COMPANY_ID: u16 = 0x009E;

/// Hard ceiling on capture length, so a forgotten session cannot hold the
/// vendor channel indefinitely.
#[cfg(windows)]
const DEFAULT_MAX_SECONDS: u64 = 600;

fn main() {
    #[cfg(not(windows))]
    {
        eprintln!("This tool requires Windows.");
        std::process::exit(1);
    }

    #[cfg(windows)]
    run();
}

#[cfg(windows)]
fn run() {
    let args: Vec<String> = std::env::args().skip(1).collect();
    if args.iter().any(|a| a == "-h" || a == "--help") {
        print_help();
        return;
    }
    let max_seconds = args
        .iter()
        .position(|a| a == "--seconds")
        .and_then(|i| args.get(i + 1))
        .and_then(|s| s.parse::<u64>().ok())
        .unwrap_or(DEFAULT_MAX_SECONDS);

    println!("Bose QC Control Center — RFCOMM listen-only capture");
    println!("====================================================\n");
    println!("This tool SENDS NOTHING. It opens the vendor RFCOMM service and");
    println!("records only what the headphones volunteer.\n");
    println!("WARNING: while this capture holds the vendor channel, Bose Music");
    println!("         (phone or desktop) may be unable to connect. Audio should");
    println!("         be unaffected. Quit with 'q' to release it immediately.\n");

    // --- locate the device -------------------------------------------------
    let Some((name, address_hex)) = find_bose_device() else {
        eprintln!("No Bose device found.\n");
        eprintln!("Checked every paired Bluetooth device for SIG company id 0x009E.");
        eprintln!("Power the headphones on and connect them to this PC, then retry.");
        std::process::exit(1);
    };

    let Some(address) = rfcomm::parse_address(&address_hex) else {
        eprintln!("Could not parse the device address.");
        std::process::exit(1);
    };

    println!("Device   : {name}");
    println!("Service  : {VENDOR_RFCOMM_UUID_TEXT}");
    println!("Max time : {max_seconds}s\n");

    // --- open --------------------------------------------------------------
    print!("Opening the vendor RFCOMM channel... ");
    let _ = std::io::stdout().flush();

    let channel = match rfcomm::ListenOnlyChannel::open(address, VENDOR_RFCOMM_UUID) {
        Ok(c) => {
            println!("connected.\n");
            c
        }
        Err(e) => {
            println!("failed.\n");
            eprintln!("{e}\n");
            eprintln!("Stopping rather than retrying. Nothing was sent to the device.");
            eprintln!("A single attempt was made and no reconnection will occur.");
            std::process::exit(2);
        }
    };
    channel.set_read_timeout(Duration::from_millis(250));

    let started = Instant::now();
    let events: Arc<Mutex<Vec<Event>>> = Arc::new(Mutex::new(Vec::new()));
    let stop = Arc::new(AtomicBool::new(false));

    events.lock().unwrap().push(Event::Note {
        at_ms: 0,
        timestamp: now_rfc3339(),
        text: format!("Channel opened to {name} on {VENDOR_RFCOMM_UUID_TEXT}. Zero bytes sent."),
    });

    // --- receive loop ------------------------------------------------------
    let rx_events = Arc::clone(&events);
    let rx_stop = Arc::clone(&stop);
    let rx = std::thread::spawn(move || {
        while !rx_stop.load(Ordering::Relaxed) {
            if started.elapsed().as_secs() >= max_seconds {
                println!("\n[time limit reached — stopping]");
                rx_stop.store(true, Ordering::Relaxed);
                break;
            }
            match channel.receive() {
                Ok(rfcomm::RecvOutcome::Data(bytes)) => {
                    let at_ms = started.elapsed().as_millis();
                    let hex = capture::to_hex(&bytes);
                    println!(
                        "  [{:>7}ms] RX {:>3} bytes  {}",
                        at_ms,
                        bytes.len(),
                        truncate(&hex, 60)
                    );
                    rx_events.lock().unwrap().push(Event::Frame {
                        at_ms,
                        timestamp: now_rfc3339(),
                        direction: "device-to-host",
                        service_uuid: VENDOR_RFCOMM_UUID_TEXT.to_string(),
                        length: bytes.len(),
                        ascii: capture::to_ascii(&bytes),
                        hex,
                    });
                }
                Ok(rfcomm::RecvOutcome::Idle) => {}
                Ok(rfcomm::RecvOutcome::Closed) => {
                    println!("\n[device closed the channel]");
                    rx_events.lock().unwrap().push(Event::Note {
                        at_ms: started.elapsed().as_millis(),
                        timestamp: now_rfc3339(),
                        text: "Device closed the RFCOMM channel.".to_string(),
                    });
                    rx_stop.store(true, Ordering::Relaxed);
                    break;
                }
                Err(e) => {
                    println!("\n[receive error: {e}]");
                    rx_stop.store(true, Ordering::Relaxed);
                    break;
                }
            }
        }
        // Dropping the channel closes the socket and releases the service.
        drop(channel);
    });

    // --- marker input ------------------------------------------------------
    println!("Capture running. Perform an action on the headphones, then press");
    println!("its key here and Enter.\n");
    println!("{KEY_HELP}\n");

    let stdin = std::io::stdin();
    for line in stdin.lock().lines() {
        if stop.load(Ordering::Relaxed) {
            break;
        }
        let Ok(line) = line else { break };
        let key = line.trim().to_lowercase();
        if key == "q" {
            break;
        }
        match Action::from_key(&key) {
            Some(action) => {
                let at_ms = started.elapsed().as_millis();
                println!("  [{at_ms:>7}ms] MARK {}", action.label());
                events.lock().unwrap().push(Event::Marker {
                    at_ms,
                    timestamp: now_rfc3339(),
                    action,
                });
            }
            None if key.is_empty() => {}
            None => println!("  (unrecognised key '{key}')\n{KEY_HELP}"),
        }
    }

    stop.store(true, Ordering::Relaxed);
    let _ = rx.join();
    println!("\nChannel closed. Nothing was sent to the device at any point.\n");

    // --- export ------------------------------------------------------------
    let events = events.lock().unwrap().clone();
    export(&events, &name);
}

#[cfg(windows)]
fn truncate(s: &str, n: usize) -> String {
    if s.len() <= n {
        s.to_string()
    } else {
        format!("{}…", &s[..n])
    }
}

/// Finds the first paired device whose profile nodes carry Bose's SIG id.
#[cfg(windows)]
fn find_bose_device() -> Option<(String, String)> {
    for dev in pnp::enumerate_bluetooth_devices() {
        if !dev.is_top_level() {
            continue;
        }
        if dev.vendor_id != Some(BOSE_SIG_COMPANY_ID) {
            continue;
        }
        let name = dev
            .friendly_name
            .clone()
            .unwrap_or_else(|| "Bose device".to_string());
        let address = pnp::device_address(&dev.instance_id)?;
        return Some((name, address));
    }
    None
}

#[cfg(windows)]
fn export(events: &[Event], device_name: &str) {
    let patterns = capture::analyse(events);

    let frames = events
        .iter()
        .filter(|e| matches!(e, Event::Frame { .. }))
        .count();
    let markers = events
        .iter()
        .filter(|e| matches!(e, Event::Marker { .. }))
        .count();

    // Raw log, one JSON object per line.
    let jsonl: String = events
        .iter()
        .filter_map(|e| serde_json::to_string(e).ok())
        .collect::<Vec<_>>()
        .join("\n");
    let _ = std::fs::write("rfcomm-capture.jsonl", jsonl);

    let mut txt = String::new();
    txt.push_str("BOSE RFCOMM LISTEN-ONLY CAPTURE\n");
    txt.push_str("===============================\n\n");
    txt.push_str(&format!("Device       : {device_name}\n"));
    txt.push_str(&format!("Service      : {VENDOR_RFCOMM_UUID_TEXT}\n"));
    txt.push_str(&format!("Generated    : {}\n", now_rfc3339()));
    txt.push_str("Posture      : LISTEN-ONLY — 0 bytes transmitted\n");
    txt.push_str(&format!("Frames       : {frames}\n"));
    txt.push_str(&format!("Markers      : {markers}\n\n"));

    txt.push_str("TIMELINE\n--------\n");
    for e in events {
        match e {
            Event::Frame {
                at_ms, length, hex, ascii, ..
            } => txt.push_str(&format!(
                "  {at_ms:>7}ms  RX  {length:>3}B  {hex}  |{ascii}|\n"
            )),
            Event::Marker { at_ms, action, .. } => {
                txt.push_str(&format!("  {at_ms:>7}ms  ACTION  {}\n", action.label()))
            }
            Event::Note { at_ms, text, .. } => {
                txt.push_str(&format!("  {at_ms:>7}ms  NOTE  {text}\n"))
            }
        }
    }

    txt.push_str("\nFRAME PATTERNS\n--------------\n");
    if patterns.is_empty() {
        txt.push_str("  No frames were received. The device volunteered nothing.\n");
    } else {
        for p in &patterns {
            txt.push_str(&format!(
                "\n  {}  ({} bytes, seen {}x)\n",
                p.hex, p.length, p.occurrences
            ));
            if p.unprompted > 0 {
                txt.push_str(&format!(
                    "      {}x with no marked action in the preceding {}ms\n",
                    p.unprompted,
                    capture::CORRELATION_WINDOW_MS
                ));
            }
            for (action, n) in &p.seen_after {
                txt.push_str(&format!("      {n}x within 2s after {action}\n"));
            }
        }
    }

    txt.push_str(
        "\nINTERPRETATION\n--------------\n\
         None. This report records temporal correlation only.\n\n\
         A frame arriving after an action is not proof that the frame encodes\n\
         that action, that the field which changed is the mode, or that the\n\
         device would accept the same bytes as a command. Establishing any of\n\
         that requires further observation, and writing to the device is a\n\
         separate decision that has not been taken.\n\n\
         Capability statuses must NOT be changed on the strength of this file\n\
         alone.\n",
    );

    let _ = std::fs::write("rfcomm-capture.txt", &txt);

    println!("Frames received : {frames}");
    println!("Actions marked  : {markers}");
    println!("\nWrote rfcomm-capture.jsonl");
    println!("Wrote rfcomm-capture.txt");

    if frames == 0 {
        println!("\nThe device sent nothing while connected. That is a real result:");
        println!("this channel may only speak when spoken to. It does not mean the");
        println!("capture failed.");
    }
}

#[cfg(windows)]
fn now_rfc3339() -> String {
    time::OffsetDateTime::now_utc()
        .format(&time::format_description::well_known::Rfc3339)
        .unwrap_or_else(|_| "unknown".to_string())
}

#[cfg(windows)]
fn print_help() {
    println!(
        "bose-rfcomm-listen — listen-only RFCOMM capture\n\n\
         Opens the Bose vendor RFCOMM service and records what the device\n\
         volunteers. Transmits nothing.\n\n\
         Options:\n  \
         -h, --help          Show this help\n  \
         --seconds <n>       Maximum capture length (default {DEFAULT_MAX_SECONDS})\n\n\
         Writes rfcomm-capture.jsonl and rfcomm-capture.txt to the working directory."
    );
}
