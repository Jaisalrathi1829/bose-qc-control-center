# Protocol Notes

**Status: first hardware observation complete. No vendor protocol has been
spoken to — the observations below are read-only and nothing was sent to the
device.**

## Observed device — 2026-08-08

A real Bose QuietComfort, renamed by its owner to **"Aurora"**, connected to
the development machine over Bluetooth Classic.

| Property | Observed value |
| --- | --- |
| SIG company id | `0x009E` (Bose Corporation), on every profile child node |
| Product id | `0x4075` |
| Transport | Bluetooth Classic only — **no `BTHLE` node exists** |
| Battery | **30%**, via the Windows PnP battery property |
| Audio endpoints | `Headphones (Aurora)` (A2DP), `Headset (Aurora Hands-Free)` (HFP) |

### Services Windows enumerated

| UUID | Service |
| --- | --- |
| `0000110B` | A2DP Audio Sink |
| `0000110C` | AVRCP Target |
| `0000110E` | AVRCP Controller |
| `0000111E` | Handsfree (HFP) |
| `{9B26D8C0-A8ED-440B-95B0-C4714A518BCC}` | **Vendor-specific RFCOMM** (`SRfcomm`) |
| `{00000000-DECA-FADE-DECA-DEAFDECACAFF}` | Windows placeholder for a service it has no driver for |

### What this establishes, and what it does not

**Establishes:**

* The device is identifiable by vendor id rather than name. This matters — the
  unit is renamed "Aurora" and matches none of the Bose name hints, so
  name-based detection found nothing at all. Vendor id detection found it
  immediately.
* Battery is readable, but **only from the HFP child node**. The top-level
  `BTHENUM\DEV_...` node reports no battery. Code that reads only top-level
  nodes — as this project's did until this session — misses it entirely.
* A vendor-specific RFCOMM service exists and is almost certainly the control
  channel Bose Music uses.

**Does not establish:** anything about what that RFCOMM channel accepts. No
frame format, no opcode, no command has been observed, because nothing was
sent. Noise control and EQ remain `UNKNOWN`, not `SUPPORTED` — the presence of
a channel is not evidence of what travels over it.

## Experiment 1 — listen-only RFCOMM, half-closed transmit

**2026-08-08T11:55:48Z. Zero bytes transmitted.**

| | |
| --- | --- |
| Service | `{9B26D8C0-A8ED-440B-95B0-C4714A518BCC}` |
| Connect | **Succeeded** |
| Frames received | **0** |
| Channel lifetime | **16 ms**, closed by the device |

### What this establishes

The vendor RFCOMM service is real, is advertised over SDP, and **accepts
inbound connections**. Windows resolved the channel number itself and the
connect completed. That is now confirmed rather than inferred.

### What it does not establish, and why

The 16 ms close is **confounded by the tool's own behaviour** and must not be
read as device policy.

The first build called `shutdown(SD_SEND)` immediately after connecting, as an
extra OS-level guarantee against transmitting. That call announces to the peer
that this side will never send anything. A device speaking a request/response
protocol has no reason to hold a channel open for a peer that has declared it
will never ask a question — so hanging up is the reasonable thing for it to do.

In other words the experiment measured our own end-of-stream signal, not the
headphones' behaviour toward a silent-but-open peer. 16 ms is far too fast to
be an idle timeout, which supports this reading.

`shutdown(SD_SEND)` is now opt-in (`--half-close`) and off by default. The
no-write guarantee is unaffected: transmitting requires a `send` call, and no
such symbol is imported anywhere in the crate.

### Not yet established

Whether the device volunteers anything to a silent peer that has *not*
announced end-of-stream. Experiment 2 repeats the capture without the
half-close.

Nothing about ANC, Aware, EQ or any other feature. A channel accepting a
connection says nothing about what travels over it.

## Nothing below here is a finding

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
