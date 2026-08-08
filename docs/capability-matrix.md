# Capability Matrix

The authoritative record of what this application can actually do.

**Last updated:** 2026-08-08
**Hardware status:** A real Bose QuietComfort (renamed "Aurora", SIG vendor
`0x009E`, product `0x4075`) has been observed connected over Bluetooth Classic.
Read-only observation only — nothing has been sent to the device.

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
| Noise control (Quiet) | Bose vendor protocol | UNKNOWN | No |
| Aware mode | Bose vendor protocol | UNKNOWN | No |
| Custom noise control | Bose vendor protocol | UNKNOWN | No |
| Equalizer | Bose vendor protocol | UNKNOWN | No |
| Multipoint | Bose vendor protocol | UNKNOWN | No |
| Firmware version | Bose vendor protocol | UNKNOWN | No |
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

### Vendor protocol features — why all `UNKNOWN`

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
