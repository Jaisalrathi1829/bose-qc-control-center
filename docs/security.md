# Security

## Principles

1. The frontend is treated as untrusted. It receives the minimum capability
   needed to render.
2. Nothing leaves the machine. Ever.
3. Device identifiers are not exposed, even to the local UI.
4. The application does not write to Bluetooth hardware at all in this build.

## No arbitrary Bluetooth writes

The command enum in `device/command.rs` is the complete set of operations the UI
can request. It contains no variant carrying a UUID, a characteristic handle, a
service identifier, or a byte array.

This is enforced by construction, and additionally pinned by a test —
`command_json_surface_contains_no_raw_byte_payloads` — which serializes command
samples and asserts the JSON contains no `uuid`, `bytes` or `payload` key. A
future contributor adding a raw-payload variant has to break that test.

Every command is validated in `DeviceCommand::validate()` before any backend
sees it, for both mock and real paths, so the mock exercises the same checks as
hardware.

## Tauri permissions

`capabilities/default.json` grants `core:default`, a few explicit window
operations, and notifications. It does not grant shell access, filesystem
access, or HTTP.

The asset protocol is disabled in `tauri.conf.json`, and the corresponding
`protocol-asset` crate feature was removed rather than enabling the protocol to
match it.

`freezePrototype` is on. The CSP forbids everything except self, with
`object-src 'none'`, `base-uri 'none'` and `form-action 'none'`. `connect-src`
permits only Tauri's IPC — there is no origin the app could reach even if a
network call were introduced.

## Privacy

| Data | Handling |
| --- | --- |
| Bluetooth address / PnP instance id | Stays in the native layer. Never sent to the UI. |
| Device id shown in UI and reports | Per-installation salted SHA-256 prefix |
| Diagnostic captures | Local files, git-ignored |
| Telemetry / analytics | None |
| Network requests | None |
| User accounts | None |

`util::stable_id()` is covered by `stable_id_does_not_leak_the_raw_identifier`,
which asserts the raw address does not appear in the output.

`.gitignore` excludes `device-report.json`, `device-report.txt`, `captures/`,
`reports/` and `*.btcapture` so device data cannot be committed by accident.

## Logging

Structured logging with explicit levels. Ordinary logs record command *names*
only — `DeviceCommand::name()` deliberately returns a static string with no
payload detail, so parameters cannot reach normal logs. Detailed diagnostic
logging is opt-in.

Never logged: credentials, tokens, secrets, raw Bluetooth addresses.

## Hardware safety

The read-only posture is a security property as much as a correctness one.
Until a protocol is verified:

- No writes are sent to any Bluetooth device.
- `set_noise_control` and `set_equalizer` return `Unsupported` with a reason.
- No firmware operations of any kind exist in the codebase.
- No fuzzing, brute-forcing, or authentication bypass.
