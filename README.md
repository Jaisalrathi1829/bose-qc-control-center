# Bose QC Control Center

A fully local Windows control center for Bose QuietComfort headphones. No cloud
services, no accounts, no telemetry, no network requests.

> **Current state: pre-hardware.** The Bose headphones have not been available
> during development, so **no Bose-specific capability has been verified**. The
> application runs against a clearly-labelled simulated device, and reads what
> standard Windows interfaces can tell it about real paired devices. Nothing is
> presented as working that has not been demonstrated.

## The guiding rule

> Build what can be proven. Simulate only what is explicitly simulated. Never pretend.

Every hardware-facing feature carries a capability status — `unknown`,
`supported`, `verified`, `experimental` or `unsupported` — and the UI renders
itself from that model. A control is never shown as functional because a
component exists to draw it.

`verified` means one thing only: the physical device was tested and demonstrated
the behaviour. It is unreachable in code without a `HardwareProof`, which can
only be constructed from an observed *change* in device-reported state. A moved
slider, an accepted command, or a changed local variable cannot produce one.

## Requirements

- Windows 10 1809+ or Windows 11
- WebView2 Runtime (ships with Windows 11)
- Node.js 20+
- Rust stable with the MSVC toolchain
- VS Build Tools with the C++ workload and a Windows SDK

## Getting started

Install dependencies:

```bash
npm install
```

Run the desktop application:

```bash
npm run tauri:dev
```

Run the frontend alone in a browser (simulated device, no native layer):

```bash
npm run dev
```

## Tests

```bash
npm run rust:test
```

```bash
npm run test:run
```

## Project layout

```
app/
  frontend/       React + TypeScript + Tailwind UI
  src-tauri/      Rust native layer
    bluetooth/    Windows PnP + radio access (read-only)
    bose/         Mock and real device backends
    device/       Capability engine, typed commands, state
tools/
  bose-discovery/ Standalone read-only discovery tool
docs/             Architecture, capability matrix, protocol notes
```

## Security posture

The frontend has no Bluetooth surface. It cannot express an arbitrary write:
there is no command variant carrying a UUID, characteristic handle, or byte
array, and a test asserts that the serialized command surface contains none.
All hardware operations pass through a typed, validated, allowlisted native
layer.

Bluetooth addresses never leave the native layer. Anything user-visible or
exportable carries a per-installation salted hash instead, so a shared
diagnostics report cannot be traced back to specific hardware.

See [docs/security.md](docs/security.md).

## Documentation

| Document | Contents |
| --- | --- |
| [environment.md](docs/environment.md) | Measured state of the development machine |
| [architecture.md](docs/architecture.md) | Layering and design decisions |
| [capability-matrix.md](docs/capability-matrix.md) | What actually works, and what does not |
| [protocol-notes.md](docs/protocol-notes.md) | Vendor protocol investigation status |
| [security.md](docs/security.md) | Threat model and constraints |
| [troubleshooting.md](docs/troubleshooting.md) | Common problems |
