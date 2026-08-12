//! Noise-control probe.
//!
//! Reads the current noise-control mode, optionally changes it, and verifies
//! the result by reading back. Every transmitted byte sequence is an exact
//! replay of traffic observed from Bose Music — see `frames.rs`.
//!
//! The verification discipline matters more than the command: a mode is only
//! reported as changed when the device itself reports the new value, and the
//! value it reports differs from what it reported before. "We sent the bytes"
//! is not success.

#[cfg(windows)]
#[path = "../../../app/src-tauri/src/bluetooth/pnp.rs"]
mod pnp;

mod frames;

#[cfg(windows)]
mod channel;

#[cfg(windows)]
use channel::{RecvOutcome, VendorChannel};
use frames::{find_reported_mode, to_hex, NoiseMode, READ_CURRENT_MODE};
#[cfg(windows)]
use std::io::Write;
#[cfg(windows)]
use std::time::{Duration, Instant};

#[cfg(windows)]
const VENDOR_RFCOMM_UUID: windows::core::GUID =
    windows::core::GUID::from_u128(0x9B26D8C0_A8ED_440B_95B0_C4714A518BCC);

#[cfg(windows)]
const BOSE_SIG_COMPANY_ID: u16 = 0x009E;

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

    // The first non-flag argument is the mode. No mode means read-only.
    let positional: Vec<&String> = {
        let mut out = Vec::new();
        let mut skip_next = false;
        for a in &args {
            if skip_next {
                skip_next = false;
                continue;
            }
            if a == "--channel" {
                skip_next = true;
                continue;
            }
            if a.starts_with("--") {
                continue;
            }
            out.push(a);
        }
        out
    };

    let requested = match positional.first() {
        None => None,
        Some(a) => match NoiseMode::parse(a) {
            Some(m) => Some(m),
            None => {
                eprintln!("Unknown mode '{a}'. Valid: quiet, aware, home.");
                std::process::exit(1);
            }
        },
    };

    println!("Bose QC Control Center — noise control probe");
    println!("============================================\n");

    let Some((name, address_hex)) = find_bose_device() else {
        eprintln!("No Bose device found (looked for SIG company id 0x009E).");
        eprintln!("Power the headphones on and connect them to this PC.");
        std::process::exit(1);
    };
    let Some(address) = channel::parse_address(&address_hex) else {
        eprintln!("Could not parse the device address.");
        std::process::exit(1);
    };

    println!("Device : {name}");
    match requested {
        None => println!("Action : read current mode only (nothing will be changed)\n"),
        Some(m) => println!("Action : set mode to {}\n", m.name()),
    }

    // `--channel N` connects directly to an RFCOMM server channel instead of
    // resolving one from the service UUID. The snoop capture put the vendor
    // traffic on DLCI 16, i.e. server channel 8.
    let explicit_channel = args
        .iter()
        .position(|a| a == "--channel")
        .and_then(|i| args.get(i + 1))
        .and_then(|s| s.parse::<u32>().ok());

    let target = match explicit_channel {
        Some(c) => {
            println!("Target : RFCOMM server channel {c} (direct)\n");
            channel::Target::Channel(c)
        }
        None => channel::Target::Service(VENDOR_RFCOMM_UUID),
    };

    print!("Opening vendor RFCOMM channel... ");
    let _ = std::io::stdout().flush();
    // `--insecure` skips the authenticated/encrypted link request, for
    // comparison against the default.
    let secure = !args.iter().any(|a| a == "--insecure");
    if secure {
        println!("Link   : requesting authenticated + encrypted\n");
    }

    let ch = match VendorChannel::open_target(address, target, secure) {
        Ok(c) => {
            println!("connected.\n");
            c
        }
        Err(e) => {
            println!("failed.\n");
            eprintln!("{e}");
            std::process::exit(2);
        }
    };
    ch.set_read_timeout(Duration::from_millis(400));

    // --- open the session ---
    //
    // The device ignored a bare mode read on an otherwise silent channel.
    // Bose Music always opens with a version request and enumerates a
    // function group before using it, so both are replayed here. Neither
    // changes any device state.
    println!("Opening session (replaying Bose Music's opening frames):");
    handshake(&ch);

    // --- read the mode before doing anything ---
    //
    // `--force` sends the change even when the device never reported a mode.
    // The frame still goes out; what is lost is the ability to say whether it
    // did anything, so the result is reported as UNVERIFIABLE rather than as
    // success. Sending blind and calling it success is the one outcome this
    // tool will not produce.
    let force = args.iter().any(|a| a == "--force");

    let before = match query_mode(&ch, "BEFORE") {
        Some(m) => Some(m),
        None if force => {
            println!("\n  Device did not report a mode. --force given, sending anyway.");
            println!("  The result will NOT be verifiable.\n");
            None
        }
        None => {
            eprintln!("\nThe device did not report a current mode. Stopping without");
            eprintln!("sending a change — there would be no way to verify the result.");
            eprintln!("Pass --force to send it regardless (result will be unverifiable).");
            std::process::exit(3);
        }
    };
    if let Some(b) = before {
        println!(
            "\n  Current mode: 0x{:02X} ({})\n",
            b,
            NoiseMode::from_index(b).map(|m| m.name()).unwrap_or("unnamed")
        );
    }

    let Some(target) = requested else {
        println!("Read-only run complete. Nothing was changed.");
        return;
    };

    if before == Some(target.index()) {
        println!(
            "Already in {}. Nothing to do — not sending a redundant change.",
            target.name()
        );
        return;
    }

    // --- send the change ---
    let frame = frames::set_mode(target);
    match ch.send_frame(&frame) {
        Ok(n) => println!(
            "  TX  {}   (set mode to {}) [{n}/{} bytes accepted]",
            to_hex(&frame),
            target.name(),
            frame.len()
        ),
        Err(e) => {
            eprintln!("\n{e}");
            std::process::exit(4);
        }
    }
    drain(&ch, Duration::from_millis(2500), "  RX  ");

    // --- read back and verify ---
    println!();
    let after = match query_mode(&ch, "AFTER") {
        Some(m) => m,
        None => {
            println!("\nRESULT: UNVERIFIABLE");
            println!("        The frame was transmitted and the stack accepted every byte.");
            println!("        The device reported nothing back, so this tool cannot tell");
            println!("        you whether anything changed.");
            println!();
            println!("        Look at your headphones. If the mode changed, the frame is");
            println!("        correct and only the device's replies are missing. If it did");
            println!("        not, the channel is not accepting commands from us at all.");
            std::process::exit(5);
        }
    };

    println!(
        "\n  Mode after: 0x{:02X} ({})\n",
        after,
        NoiseMode::from_index(after)
            .map(|m| m.name())
            .unwrap_or("unnamed")
    );

    // The verification rule used throughout this project: the state must have
    // changed, and it must have changed to what was asked for.
    if after != target.index() {
        println!("RESULT: NOT VERIFIED — the device reports a mode we did not ask for.");
        println!(
            "        Asked for 0x{:02X}, device reports 0x{:02X}.",
            target.index(),
            after
        );
        std::process::exit(7);
    }

    match before {
        Some(b) if b == after => {
            println!("RESULT: NOT VERIFIED — the device reports the same mode as before.");
            println!("        The command was transmitted but had no observable effect.");
            std::process::exit(6);
        }
        Some(b) => {
            println!("RESULT: VERIFIED");
            println!(
                "        Device-reported mode changed 0x{b:02X} -> 0x{after:02X}, matching the request."
            );
            println!(
                "        {} -> {}",
                NoiseMode::from_index(b).map(|m| m.name()).unwrap_or("unnamed"),
                target.name()
            );
        }
        None => {
            // --force path: the device answered afterwards but not before, so
            // there is no baseline to compare against. It matches the request,
            // which is suggestive, but "changed" cannot be claimed.
            println!("RESULT: PARTIAL — device now reports the requested mode,");
            println!("        but it reported nothing beforehand, so there is no baseline");
            println!("        and no proof this command caused the change.");
            std::process::exit(8);
        }
    }
}

/// Replays the session-opening frames Bose Music sends. Read-only.
#[cfg(windows)]
fn handshake(ch: &VendorChannel) {
    for (frame, label) in [
        (&frames::PROTOCOL_VERSION_REQUEST[..], "protocol version"),
        (&frames::NOISE_GROUP_ENUMERATE[..], "enumerate noise group"),
    ] {
        match ch.send_frame(frame) {
            Ok(n) => println!("  TX  {}   ({label}) [{n}/{} bytes accepted]", to_hex(frame), frame.len()),
            Err(e) => {
                println!("  [send failed: {e}]");
                return;
            }
        }
        drain(ch, Duration::from_millis(1500), "  RX  ");
    }
}

/// Sends the read frame and returns the mode the device reports.
#[cfg(windows)]
fn query_mode(ch: &VendorChannel, label: &str) -> Option<u8> {
    match ch.send_frame(&READ_CURRENT_MODE) {
        Ok(n) => println!(
            "  TX  {}   (read current mode) [{label}] [{n}/{} bytes accepted]",
            to_hex(&READ_CURRENT_MODE),
            READ_CURRENT_MODE.len()
        ),
        Err(e) => {
            println!("  [send failed: {e}]");
            return None;
        }
    }

    let deadline = Instant::now() + Duration::from_millis(3000);
    let mut found = None;
    while Instant::now() < deadline {
        match ch.receive() {
            Ok(RecvOutcome::Data(bytes)) => {
                println!("  RX  {}", to_hex(&bytes));
                if found.is_none() {
                    found = find_reported_mode(&bytes);
                }
            }
            Ok(RecvOutcome::Idle) => {
                if found.is_some() {
                    break;
                }
            }
            Ok(RecvOutcome::Closed) => {
                println!("  [device closed the channel]");
                break;
            }
            Err(e) => {
                println!("  [receive error: {e}]");
                break;
            }
        }
    }
    found
}

/// Reads and prints whatever arrives for a while, without interpreting it.
#[cfg(windows)]
fn drain(ch: &VendorChannel, how_long: Duration, prefix: &str) {
    let deadline = Instant::now() + how_long;
    while Instant::now() < deadline {
        match ch.receive() {
            Ok(RecvOutcome::Data(bytes)) => println!("{prefix}{}", to_hex(&bytes)),
            Ok(RecvOutcome::Idle) => {}
            Ok(RecvOutcome::Closed) => {
                println!("  [device closed the channel]");
                break;
            }
            Err(_) => break,
        }
    }
}

#[cfg(windows)]
fn find_bose_device() -> Option<(String, String)> {
    for dev in pnp::enumerate_bluetooth_devices() {
        if !dev.is_top_level() || dev.vendor_id != Some(BOSE_SIG_COMPANY_ID) {
            continue;
        }
        let name = dev.friendly_name.clone().unwrap_or_else(|| "Bose device".into());
        let address = pnp::device_address(&dev.instance_id)?;
        return Some((name, address));
    }
    None
}

#[cfg(windows)]
fn print_help() {
    println!(
        "bose-anc-probe — read or set the noise-control mode\n\n\
         Usage:\n  \
         bose-anc-probe              Read the current mode. Changes nothing.\n  \
         bose-anc-probe quiet        Set to Quiet\n  \
         bose-anc-probe aware        Set to Aware\n  \
         bose-anc-probe home         Set to Home\n\n\
         Every transmitted byte sequence is an exact replay of traffic observed\n\
         from Bose Music. A change is reported as verified only when the device\n\
         itself reports the new mode AND that differs from what it reported\n\
         before."
    );
}
