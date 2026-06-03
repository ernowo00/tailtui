# Product Spec: tailscale-tui v1 (SLC)

## 1) Product Goal

Build a simple and lovable terminal UI for monitoring local Tailscale status and toggling connectivity (`up` / `down`) through the `tailscale` CLI.

## 2) Target Users

- Developers and power users working in a terminal-first workflow.
- Users who want quick visibility and control without opening web/admin UI.

## 3) Core User Stories

1. As a user, I can open `tailscale-tui` and immediately see my current connection status.
2. As a user, I can refresh status data with one key.
3. As a user, I can trigger `tailscale up` when disconnected.
4. As a user, I can trigger `tailscale down` when connected.
5. As a user, I can navigate all focusable UI sections with Vim-style keys.
6. As a user, I can see command success/failure feedback in the UI.

## 4) Scope (v1)

### In Scope

- Read-only status rendering from CLI:
  - Backend state (running/stopped)
  - Tailnet name (if available)
  - Device name (if available)
  - Tailscale IP addresses (if available)
  - Exit node usage summary (if available)
- Actions:
  - Trigger `tailscale up`
  - Trigger `tailscale down`
  - Refresh status
- Vim-style navigation and action controls.
- Error messaging when command execution fails.

### Out of Scope

- Login/onboarding or auth browser flow handling.
- Config editor for advanced flags.
- API calls to Tailscale control plane.
- Background daemon/process supervisor functionality.

## 5) UX Principles

- Zero-learning default: obvious status + obvious actions.
- Fast keyboard flow: no mouse required.
- Predictable feedback: every action updates status and shows result.
- Fail loud, recover fast: clear errors and simple retry.

## 6) Runtime Constraints

- Must run on Linux terminal with Rust binary.
- Must require `tailscale` CLI available in `PATH`.
- Must not require root privileges by default (depends on host tailscale setup).
