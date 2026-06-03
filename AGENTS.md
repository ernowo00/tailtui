# Agent guide: tailscale-tui

This document helps humans and **AI coding agents** work on this repository efficiently. Prefer **small, focused changes** that match existing patterns.

## Product constraints (non-negotiable)

- **CLI only**: integrate with Tailscale by invoking the `tailscale` binary. **Do not** add HTTP clients or Tailscale control-plane APIs.
- **KISS / SOLID**: keep modules single-purpose; UI must not spawn processes directly—go through the infra/service layer.
- **SLC scope**: monitoring + `up` / `down` + machine list + copy IPv4. Avoid scope creep (ACL editor, login flows, etc.) unless the user explicitly asks.

Canonical contracts live in `specs/`, especially `specs/cli-contract.md` and `specs/architecture-contract.md`.

## Architecture (where to edit what)

| Layer | Path | Responsibility |
| --- | --- | --- |
| Composition | `src/app/run.rs` | Event loop, wiring domain + infra + UI, terminal lifecycle |
| Domain | `src/domain/` | Models, `AppError`, traits (`StatusReader`, `TailscaleController`) |
| Infra | `src/infra/cli/` | `tailscale` invocation, JSON parsing, timeouts |
| Infra | `src/infra/clipboard.rs` | Clipboard helpers (non-blocking; short wait policy) |
| UI | `src/ui/tui/render.rs` | Layout, colors, help overlay |
| UI | `src/ui/tui/event.rs` | Key → `UiEvent` mapping |

**Dependency direction**: `domain` has no infra imports. `app` composes everything.

## Commands agents should run

From repo root:

```bash
cargo check
cargo test
cargo clippy --all-targets --all-features -- -D warnings
```

Or `make check`, `make test`, `make clippy`.

**WSL2 (Tailscale on Windows)**: users without a Linux `tailscale` package should run `bash scripts/install-wsl.sh` (or `make install-wsl`) to symlink the Windows CLI and install `tailtui`—do not install Tailscale inside the distro unless they want a separate Linux daemon.

**Do not** assume `cargo run` works in headless agent runners: TUIs need a TTY. Users validate interactively on their machine.

## Tailscale CLI expectations

- Status: `tailscale status --json` — parser in `src/infra/cli/parser.rs`. Optional fields must stay tolerant.
- Actions: `tailscale up`, `tailscale down` — see `src/infra/cli/command_runner.rs`.
- Backend state strings (e.g. `Stopped`) may vary; compare case-insensitively where the UI branches on state.

## UI / UX invariants

- **Help overlay**: `?` opens; **any key** closes it and must **not** trigger other actions (handled in `src/app/run.rs`).
- **Non-focus panels** (header, feedback): yellow title + border, **white** body text.
- **Layout**: top row is **Status | Actions** (status has no border, not focusable); **Machines** is full width below (stateful `List` + `ListState` for scrolling).
- **Machines** when stopped: show instructional lines, not stale peer rows.
- **Offline** machines: dimmed (`DarkGray`) when not selected.

## Clipboard (`y`)

- Tries `wl-copy`, then `xclip`, then `xsel`.
- Uses **short timeout** after stdin close so the TUI never blocks on long-lived clipboard owners.

## Tests

- Parser and small helpers have unit tests; extend tests when changing JSON mapping or selection/copy logic.
- Avoid integration tests that require a real `tailscale` daemon in CI unless gated.

## Common pitfalls for agents

1. **Blocking the UI**: never `wait()` on clipboard or child processes without a timeout strategy consistent with `clipboard.rs`.
2. **Parsing `Peer` JSON**: keys and optional fields differ; keep extraction defensive.
3. **Focus vs global keys**: navigation keys should respect focus; help mode short-circuits the loop.
4. **Edition 2024**: keep `Cargo.toml` edition aligned with the toolchain; use idiomatic Rust 2024 where applicable.
5. **WSL + Windows Tailscale**: shell aliases do not reach `Command::new("tailscale")`; PATH must include `~/.local/bin/tailscale` (see `scripts/install-wsl.sh`).

## Documentation

- **User-facing**: `README.md`
- **Specs / contracts**: `specs/`
- **This file**: contributor and agent-oriented; keep it accurate when architecture or constraints change.

## Something agents should tell maintainers

If behavior depends on Tailscale version or desktop environment (Wayland vs X11, clipboard tools), document the assumption in code comments **only** when non-obvious—prefer updating this file or `specs/` instead of long comment threads in source.
