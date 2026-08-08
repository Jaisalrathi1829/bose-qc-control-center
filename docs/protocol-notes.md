# Protocol Notes

**Status: no vendor protocol has been investigated, because no Bose device has
been available.**

This document records what was reasoned about, so the eventual hardware session
does not start from scratch — and so nobody mistakes reasoning for findings.

## Nothing here is a finding

There are no UUIDs, opcodes, frame formats or command tables in this document
or anywhere in the codebase. That is deliberate. Publishing speculative
protocol details invites someone to implement against them, and speculative
protocol bytes sent at real hardware is exactly the risk this project refuses
to take.

When real observations exist, they will be recorded here with the capture that
produced them.

## API-first: what was ruled out before considering a vendor protocol

Per the API-first rule, standard interfaces were assessed first.

| Interface | Exposes | Covers ANC/EQ? |
| --- | --- | --- |
| A2DP | Audio streaming | No |
| AVRCP | Play/pause/next/previous, metadata | No |
| HFP / HSP | Call audio, battery indication | No |
| BLE Battery Service (0x180F) | Battery level | No |
| BLE Device Information (0x180A) | Manufacturer, model, firmware strings | No |
| BLE Generic Access (0x1800) | Device name, appearance | No |
| HID | Key/consumer controls | No |

Conclusion: **no standard Bluetooth profile exposes noise control or EQ.** Those
features are reachable only through a vendor-specific interface, which cannot be
investigated without the hardware.

Two features *are* plausibly reachable through standard interfaces, and are the
right things to implement first:

* **Battery** — via the Windows PnP property, which Windows populates from HFP
  or the BLE Battery Service. Mechanism verified on this machine against a
  different device; unverified for Bose.
* **Transport controls** — via the Windows global media session, which works
  for any device Windows routes audio to, without touching Bluetooth directly.

## Planned investigation, when hardware is present

Read-only and passive, per the agreed posture. The tool sends nothing to the
device beyond standard protocol reads.

1. Pair the QC and confirm it appears under `BTHENUM` / `BTHLE`.
2. Enumerate SDP service records — which RFCOMM channels are advertised.
3. Enumerate GATT services and characteristics, and their properties.
4. Read every readable characteristic once.
5. Subscribe to notifications and idle.
6. With capture running, physically operate the headphones — change ANC mode,
   press volume, power cycle — and record which handles emit data and when.
7. Correlate emissions against timestamped manual markers.

Step 6 is the valuable one. Passive observation of the device reporting its own
state changes is the strongest evidence available without writing anything, and
it is what turns `unknown` into a real finding.

## Explicit non-goals

The following are out of scope permanently, not merely deferred:

- Firmware modification, extraction or flashing
- Bypassing or defeating authentication, pairing or encryption
- Brute-forcing anything
- Destructive or uncontrolled fuzzing
- Sending commands whose effect is not understood

The goal is interoperability with hardware the user owns, not compromise of it.

## Stop condition

If, after the investigation above, a feature cannot be reached through a
standard Windows API, a standard Bluetooth profile, a publicly documented
interface, or safely verified observation of the user's own device, it will be
marked `UNSUPPORTED` with the reasoning recorded — and the work will stop there
rather than escalating to riskier techniques.
