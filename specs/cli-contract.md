# CLI Contract: tailscale-tui <-> tailscale binary

## 1) Hard Rules

- Only use the `tailscale` CLI binary.
- Do not call any HTTP API.
- Do not parse human-only output when JSON output exists.

## 2) Required Commands

## Read status

- Primary:
  - `tailscale status --json`
- Fallback (optional if JSON unsupported):
  - `tailscale status`

Expected behavior:

- Success exit code returns parsed `TailscaleStatus`.
- Non-zero exit code returns `CommandFailed` with stderr attached.

## Bring network up

- Command:
  - `tailscale up`

Expected behavior:

- Success returns `ActionResult { ok: true, message }`.
- Failure returns `CommandFailed`.
- After success, app should automatically refresh status.

## Bring network down

- Command:
  - `tailscale down`

Expected behavior:

- Success returns `ActionResult { ok: true, message }`.
- Failure returns `CommandFailed`.
- After success, app should automatically refresh status.

## 3) Execution Policy

- Execution timeout default: 8 seconds.
- Any timeout maps to `Timeout`.
- UTF-8 decoding errors map to `ParseFailed`.
- Command path lookup errors map to `CliNotFound`.

## 4) Parsing Contract (`status --json`)

Parse minimally required fields; ignore unknown fields:

- Backend state
- Self node hostname/name (if present)
- Tailnet name (if present)
- Tailscale IP list (if present)
- Exit node in use (if present)

Parser must be tolerant to missing optional fields.

## 5) Security and Safety

- No shell interpolation (`sh -c`) allowed.
- Use argument arrays with `std::process::Command`.
- Do not log sensitive tokens.
