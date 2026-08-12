# Capability Matrix

The authoritative record of what this application can actually do.

**Last updated:** 2026-08-12
**Hardware status:** A real Bose QuietComfort (renamed "Aurora", SIG vendor
`0x009E`, product `0x4075`) has been observed connected over Bluetooth Classic.
Battery, device identity and Windows volume are verified. Vendor-protocol
frames (noise control) have been transmitted — see Experiment 4 in
[protocol-notes.md](protocol-notes.md) — but produced no observable device
change, so those capabilities remain `SUPPORTED`, not `VERIFIED`.

## Status definitions

| Status | Meaning |
| --- | --- |
| `UNKNOWN` | We have not established whether the feature is accessible. |
| `SUPPORTED` | A technically valid interface appears to expose it, but it has **not** been confirmed on the physical device. |
| `VERIFIED` | The actual physical device was tested and the feature was confirmed to work. |
| `EXPERIMENTAL` | Evidence suggests it may work, but verification is incomplete. |
| `UNSUPPORTED` | Cannot currently be accessed safely through available interfaces. |

`SUPPORTED` is never treated as working. The UI renders it with an explicit
caveat, and `CapabilityStatus::is_actionable()` returns `false` for it.

## Matrix

| Feature | Mechanism | Status | Hardware verified |
| --- | --- | --- | --- |
| Device detection | Windows PnP, SIG vendor id `0x009E` | **VERIFIED** | **Yes** |
| Connection state | Windows PnP property | **VERIFIED** | **Yes** |
| Battery | Windows PnP battery property (HFP child node) | **VERIFIED** | **Yes** — read 30% |
| Device identity | Windows PnP | **VERIFIED** | **Yes** |
| Windows volume | Core Audio | **VERIFIED** | **Yes** (see below) |
| Playback / transport | Windows media session (not yet wired) | UNKNOWN | No |
| Noise control (Quiet) | Vendor RFCOMM, DLCI 16, group `0x1F` | SUPPORTED | No |
| Aware mode | Vendor RFCOMM, group `0x1F`, mode `0x01` | SUPPORTED | No |
| Custom noise control | Vendor RFCOMM, group `0x1F` | UNKNOWN | No |
| Equalizer | Bose vendor protocol | UNKNOWN | No |
| Multipoint | Vendor RFCOMM, group `0x04` | UNKNOWN | No |
| Firmware version | Vendor RFCOMM, group `0x00` | SUPPORTED | No |
| Auto-off | Bose vendor protocol | UNKNOWN | No |
| Voice prompts | Bose vendor protocol | UNKNOWN | No |
| Sidetone | Bose vendor protocol | UNKNOWN | No |
| Device rename | Bose vendor protocol | UNKNOWN | No |

**No Bose-specific capability is `VERIFIED`.** That is the correct state: no
Bose device has been interrogated. The one verified entry is Windows system
volume, which is a system capability rather than a device one.

## Mechanism notes

### Windows volume — what "VERIFIED" means here, precisely

Core Audio enumeration was run against this machine's real hardware. It
returned `Speakers (Realtek(R) Audio)` at 44%, muted, default render endpoint,
48000 Hz — and a volume round-trip confirmed the endpoint reports back the
value that was set.

That verifies the **Windows audio path**, on this machine, for this endpoint.
It does **not** verify anything about a Bose endpoint, which will not exist
until the headphones are paired and connected. When they are, the same code
will read whatever endpoint Windows creates for them, and the capability will
re-verify against that.

This distinction matters and the code preserves it: `attach_windows_audio()`
builds a `HardwareProof` from the endpoint it actually read, naming that
endpoint in the evidence string.

Note also that Windows system volume for an endpoint is a different mechanism
from any volume the headphones keep internally. The UI labels it as such and
never presents the two as one control.

### Battery — now verified, and the bug that hid it

Confirmed on the physical device: **30%**, read from the Windows PnP battery
property while the headphones were connected.

The value lives on the **HFP child node** (`Aurora Hands-Free AG`,
service `0000111E`), not on the top-level `BTHENUM\DEV_...` node, which reports
nothing. The original implementation read only top-level nodes and therefore
reported "no battery reported" for a device that was in fact reporting it.
`enumerate_bluetooth_devices()` now folds properties across every node sharing
a device address.

### Device detection — why the name hint was not enough

The test unit is renamed **"Aurora"** and matches none of the Bose name hints,
so name-based detection found nothing. Every profile child node carries
`VID&0001009E` — SIG company `0x009E`, Bose Corporation — which identifies it
unambiguously. `is_bose_device()` prefers the vendor id and falls back to the
name only when no vendor id is exposed.

### Noise control — why `SUPPORTED` and not `VERIFIED`

Experiment 3 recovered the actual exchange Bose Music uses, from a phone-side
HCI snoop log. The interface is now specific rather than hypothetical: vendor
RFCOMM DLCI 16, function group `0x1F`, `1F 03 05 02 XX` to set the mode, with
`XX` = `0x00` Quiet, `0x01` Aware, `0x02` Home. Six mode changes were observed
with the device naming the resulting mode in ASCII each time.

That is a real interface, so `UNKNOWN` would understate it. But `VERIFIED` in
this project means *our own* command produced an observed state change on the
physical device, and this project has still transmitted zero bytes. The
capture is evidence about the protocol, not about our implementation — which
does not exist yet.

`SUPPORTED` is exactly the state this evidence supports: the interface appears
to expose the feature, and it has not been confirmed from our code.

### Custom noise control and multipoint — still `UNKNOWN`

Mode `0x02` is named "Home" and mode `0x03` has an empty name, which hints at
user-definable slots, but nothing in the capture shows one being created or
edited. Group `0x04` carries paired-device addresses and names, which is
suggestive for multipoint, but it has not been decoded. Neither is promoted on
a hint.

### Equalizer — still `UNKNOWN`

No equalizer traffic appears in the capture; the slider was evidently not
moved during it. Absence of evidence is not evidence of absence, so this stays
`UNKNOWN` rather than becoming `UNSUPPORTED`. A capture that exercises the EQ
would likely resolve it.

### Vendor protocol features — the original reasoning

Bose headphones expose device control through a vendor-specific protocol rather
than a standard Bluetooth profile. Standard interfaces were considered first,
per the API-first rule:

| Standard interface | Applicable? |
| --- | --- |
| A2DP | Audio streaming only. No device control. |
| AVRCP | Transport controls (play/pause/next). No ANC or EQ. |
| HFP/HSP | Call audio; carries a battery indication. No ANC or EQ. |
| BLE Battery Service (0x180F) | Battery only. |
| BLE Device Information (0x180A) | Static strings only. |
| HID | Not applicable to these controls. |

No standard interface exposes noise control or EQ. That leaves a vendor
protocol, which cannot be investigated without the physical device.

The current build therefore returns `Unsupported` with a reason from
`set_noise_control` and `set_equalizer` rather than sending speculative bytes at
the hardware. See [protocol-notes.md](protocol-notes.md).

## What changes when the headphones arrive

1. Pair the QC with this machine.
2. Run the discovery tool (read-only, passive).
3. Record what the device actually exposes.
4. Update this matrix from observation, not assumption.
5. Only then implement anything against it.
