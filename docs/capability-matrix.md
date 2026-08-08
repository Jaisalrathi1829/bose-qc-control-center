# Capability Matrix

The authoritative record of what this application can actually do.

**Last updated:** 2026-08-08
**Hardware status:** Bose QuietComfort headphones **not yet available**. No Bose
device has ever been paired with the development machine.

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
| Device detection | Windows PnP (CfgMgr32) | SUPPORTED | No |
| Connection state | Windows PnP property | SUPPORTED | No |
| Battery | Windows PnP battery property | UNKNOWN for Bose | No |
| Device identity | Windows PnP | SUPPORTED | No |
| Windows volume | Core Audio (not yet wired) | UNKNOWN | No |
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

**Nothing in this project is `VERIFIED`.** That is the correct state: no Bose
device has been interrogated.

## Mechanism notes

### Battery — why "UNKNOWN for Bose" and not "SUPPORTED"

The read mechanism is verified working on this machine: the Windows PnP battery
property returned `60` for a paired Legion mouse. But a mechanism working for
one device says nothing about another. Windows populates that property from
either the HFP battery indication or the BLE Battery Service, and whether a
Bose QC reports through either is unknown until one is present.

The code reflects this precisely. In `bose/real.rs`, battery becomes `VERIFIED`
only after an actual value has been read from the actual device
(`battery_ever_read`), and stays `SUPPORTED` at best when the device is
connected but silent. Two tests pin this behaviour:

* `battery_capability_requires_an_actual_reading`
* `battery_capability_is_verified_once_actually_read`

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
