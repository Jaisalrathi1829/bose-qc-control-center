# Architecture

## Layering

```
              React + TypeScript UI
                       │
            renders from capability model
                       │
                       ▼
              Typed command (no bytes)
                       │
                       ▼
        ┌──────── Tauri IPC boundary ────────┐
                       │
                  validate()
                       │
                       ▼
                  BoseDevice trait
                  /             \
                 ▼               ▼
          MockBoseDevice    RealBoseDevice
                                  │
                                  ▼
                       Windows PnP / CfgMgr32
                       (read-only, today)
```

The frontend never touches Bluetooth. It cannot: no command variant carries a
UUID, handle or payload, so an arbitrary write is not expressible in the type
system rather than merely discouraged.

## Key decisions

### Capability status is enforced, not documented

`CapabilityStatus::Verified` is unreachable without a `HardwareProof`. That type
has private fields and exactly two constructors:

* `HardwareProof::observed(key, before, after, expected)` — returns `None` if
  `before == after`, or if `after != expected`. So "we set a local variable and
  read it back" and "the device changed to something we did not ask for" both
  fail to produce proof.
* `HardwareProof::observed_passively(key, detail)` — for read-only capabilities
  where the device volunteered a value.

The mock backend cannot construct one at all, which is why it reports
`Experimental` and never `Verified`.

### Commands return outcomes, not unit

`set_noise_control` returns `CommandOutcome`, not `()`. A backend that cannot
confirm a state change is required by the signature to say
`SentUnverified { reason }`, which the UI renders as *"Command sent. State could
not be verified."* — deliberately not styled as success.

### The store always re-reads after a command

`deviceStore.run()` calls `refresh()` after every command. The UI never assumes
a command took effect and never optimistically updates device state; it re-reads
and renders what came back. The EQ sliders keep a local draft purely so dragging
feels responsive, and that draft is re-seeded from device state.

### Why CfgMgr32 rather than WinRT for PnP

WinRT's `DeviceInformation` exposes the same properties, but requesting extra
properties needs an `IIterable<HSTRING>`, which is awkward to construct from
Rust for no benefit at this layer. CfgMgr32 is synchronous, needs no COM
apartment, and reads the properties directly. WinRT remains the right choice for
GATT and RFCOMM when that work begins.

### Mock imperfection is intentional

The mock returns `SentUnverified` on every 7th mutation and can be configured to
fail reads. Without this, the UI's caution and error paths would never execute
until they met real hardware, which is the worst time to discover they are
wrong.

### Device identifiers are hashed

`util::stable_id()` returns a salted SHA-256 prefix. The salt is per
installation, so the same headphones produce different ids on different
machines and a shared diagnostics report cannot be correlated back to hardware.
The raw instance id is deliberately not included in the snapshot handed to the
UI, so it cannot leak into a screenshot or an export.

## Stack

| Layer | Choice | Note |
| --- | --- | --- |
| Shell | Tauri 2 | Small binary, real installer, native tray |
| UI | React 19 + TypeScript | Strict mode, no `any` in device types |
| Styling | Tailwind CSS 4 | CSS-first config in `styles.css` |
| Native | Rust | `windows` crate 0.58 |
| State | Zustand | Small, no boilerplate |
| Storage | SQLite (`rusqlite`, bundled) | Not yet wired |

## Not yet implemented

Named honestly rather than stubbed with fake controls:

- Windows Core Audio (volume, mute, endpoints)
- Windows media session transport controls
- System tray, startup, auto-connect, notifications
- SQLite persistence for settings and profiles
- The standalone discovery tool in `tools/bose-discovery/`
- Any Bose vendor protocol

The Settings and Profiles pages state plainly which controls are unavailable
rather than showing switches that do nothing.
