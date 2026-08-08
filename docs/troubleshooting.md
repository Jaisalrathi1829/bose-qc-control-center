# Troubleshooting

## Build

### `error: Missing manifest in toolchain 'stable-x86_64-pc-windows-msvc'`

The rustup toolchain registered but did not fully download. Repair it:

```bash
rustup toolchain uninstall stable-x86_64-pc-windows-msvc
```

```bash
rustup toolchain install stable-x86_64-pc-windows-msvc --profile default
```

### `link.exe not found` / `cl.exe not found`

The VS C++ workload is missing. A Visual Studio install that contains only
`Common7`, `Licenses` and `MSBuild` cannot compile native code. Add the
workload from an **elevated** shell:

```powershell
Start-Process "C:\Program Files (x86)\Microsoft Visual Studio\Installer\vs_installer.exe" -Verb RunAs -Wait -ArgumentList 'modify --installPath "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools" --add Microsoft.VisualStudio.Workload.VCTools --includeRecommended --quiet --norestart'
```

Verify:

```powershell
Get-ChildItem "C:\Program Files (x86)\Microsoft Visual Studio\2022\BuildTools\VC\Tools\MSVC" -Directory
```

### `The 'tauri' dependency features on Cargo.toml does not match the allowlist`

A crate feature is enabled that the config does not permit, or vice versa. This
project disables the asset protocol, so the `protocol-asset` feature must stay
off in `Cargo.toml`.

### Exit code 87 from `vs_installer.exe`

`ERROR_INVALID_PARAMETER`. Usually a component id that is not valid for the
installed product version, or a non-elevated shell. Drop explicit component ids
and use `--includeRecommended`, and run elevated.

## Runtime

### The app shows SIMULATED and I have real headphones

The application starts on the mock backend deliberately, so a fresh install
never silently contacts hardware. Switch in **Settings → Device Source → Real
hardware**.

### Real hardware is greyed out

You are running the browser preview (`npm run dev`), which has no native layer.
Use `npm run tauri:dev`.

### My Bose headphones are not detected

Check in order:

1. Are they paired in Windows Settings → Bluetooth & devices?
2. Are they powered on and connected to *this* machine rather than a phone?
3. Does Windows itself show them?

```powershell
Get-PnpDevice -Class Bluetooth | Where-Object Status -eq OK
```

Detection matches on the Windows friendly name. A renamed device may not match
the Bose hints in `bose/mod.rs`.

### Battery shows "Not reported"

This is honest, not broken. Windows populates the battery property from the HFP
battery indication or the BLE Battery Service. Some devices report only while
audio is streaming; some never report. The application will not invent a value.

Check what Windows itself has:

```powershell
Get-PnpDevice -Class Bluetooth | ForEach-Object { (Get-PnpDeviceProperty -InstanceId $_.InstanceId -KeyName "{104EA319-6EE2-4701-BD47-8DDBF425BBE5} 2" -EA SilentlyContinue).Data }
```

If Windows has no value, neither can this application.

### Noise control and EQ say "Not yet verified"

Correct for the current build. No Bose vendor protocol has been verified, so the
app refuses to send speculative commands. See
[protocol-notes.md](protocol-notes.md).
