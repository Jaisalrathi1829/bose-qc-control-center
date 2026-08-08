# Environment Audit

Audited 2026-08-08 on the development machine. Everything below was measured,
not assumed.

## System

| Item | Value |
| --- | --- |
| OS | Windows 11 Home Single Language |
| Version | 10.0.26200 (build 26200) |
| Architecture | AMD64 |
| Shell | PowerShell 7.6.3 |

## Toolchain

| Tool | State at audit | State after setup |
| --- | --- | --- |
| Node.js | v24.12.0 | unchanged |
| npm | 11.6.2 | unchanged |
| pnpm / yarn | absent | absent (not needed) |
| Git | 2.53.0.windows.2 | unchanged |
| Rust / rustc | **absent** | 1.97.1 (8bab26f4f 2026-07-14) |
| cargo | **absent** | 1.97.1 |
| rustup | **absent** | 1.29.0 |
| MSVC (`cl.exe`, `link.exe`) | **absent** | 14.44.35207 |
| Windows SDK | **absent** | 10.0.26100.0 |
| WebView2 Runtime | 151.0.4129.59 | unchanged |
| .NET SDK | 10.0.204 | unchanged (not used) |

### What was installed, and why

Tauri requires the MSVC toolchain on Windows; the GNU toolchain is not
officially supported for Tauri 2. Two installs were therefore necessary:

1. **Rust via winget** (`Rustlang.Rustup`) — user-scope, no elevation.
   The winget package registered the `stable-x86_64-pc-windows-msvc` toolchain
   but did not fully download it (`error: Missing manifest in toolchain`).
   Repaired with `rustup toolchain uninstall` followed by a fresh
   `rustup toolchain install stable-x86_64-pc-windows-msvc --profile default`.

2. **VC++ build tools workload** into the existing VS 2022 BuildTools install.
   Visual Studio 2022 Community and BuildTools 17.14 were both present but
   carried only `Common7`, `Licenses` and `MSBuild` — no `VC\Tools\MSVC` and no
   Windows Kits, so neither could compile C or link native code.

   The first attempt failed with exit code 87 (`ERROR_INVALID_PARAMETER`). Two
   causes: the shell is not elevated, and the explicitly named
   `Microsoft.VisualStudio.Component.Windows11SDK.22621` component was not
   valid for this install. Succeeded with an elevated
   `Start-Process -Verb RunAs` and the simpler
   `--add Microsoft.VisualStudio.Workload.VCTools --includeRecommended`, which
   pulls a matching Windows SDK automatically.

Nothing else was installed. .NET 10 is present on the machine but is not used
by this project.

## Bluetooth

| Item | Value |
| --- | --- |
| Adapter | Intel(R) Wireless Bluetooth(R) — `USB\VID_8087&PID_0033` |
| Status | OK |
| `bthserv` | Running |
| `BthAvctpSvc` | Running |
| BLE support | Present — `BTH\MS_BTHLE` (Microsoft Bluetooth LE Enumerator) |
| RFCOMM | Present — `BTH\MS_RFCOMM` |

### Paired devices at audit time

A phone, a speaker named "Aurora", a "ZEB-DUKE" audio device, and a Legion
M600s mouse.

**No Bose device is paired with this machine.** This is consistent with the
headphones being unavailable during the session, and it is why every
Bose-specific capability in this project is currently `unknown`.

## Verified mechanism: Windows PnP battery

The Windows device property
`{104EA319-6EE2-4701-BD47-8DDBF425BBE5} 2` was read against the paired devices
on this machine. It returned **60** for the Legion M600s mouse.

This verifies that:

* the property exists and is readable on this Windows build;
* the read path this project uses (CfgMgr32 `CM_Get_DevNode_PropertyW`) is the
  right mechanism for Bluetooth battery on Windows.

It does **not** verify anything about Bose headphones. Whether a Bose QC
populates this property is unknown and can only be established with the device
present. See [capability-matrix.md](capability-matrix.md).

## Audio endpoints

Only `Speakers (Realtek(R) Audio)` and `Microphone Array (Realtek(R) Audio)`
were present. No Bluetooth audio endpoint existed at audit time, because no
audio device was connected.

## Reproducing this audit

```powershell
Get-CimInstance Win32_OperatingSystem | Select-Object Caption, Version
Get-PnpDevice -Class Bluetooth | Where-Object Status -eq OK
Get-Service bthserv, BthAvctpSvc
```
