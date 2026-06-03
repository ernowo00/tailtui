# TUI Contract: navigation and interactions

## 1) Layout (v1)

Single-screen layout:

1. **Header** — app title and short help hint.
2. **Top row (split)** — **Status** (read-only, borderless) | **Actions** (focusable).
3. **Machines** — full-width peer list (focusable).
4. **Feedback** — last command / clipboard / errors.

Only **Machines** and **Actions** receive keyboard focus; **Status** is informational only.

## 2) Vim-style Keybindings

## Global

- `q`: quit
- `r`: refresh status
- `?`: toggle help overlay

## Focus navigation

- `h` / `l`: move focus **Machines** ↔ **Actions** (wraps; status is not in the focus cycle)
- `k`: move selection up
- `j`: move selection down
- `PgUp` / `PgDn`: page the **Machines** list (when that panel is focused); the list scrolls to keep the selection visible

## Action triggers

- `u`: run `tailscale up`
- `d`: run `tailscale down`
- `Enter`: activate currently focused action

## Clipboard

- `y`: copy selected peer IPv4 (requires **Machines** list data; respects stopped-state rules for peers)
- `Y` / Shift+y: copy this node’s first IPv4 from parsed `Self.TailscaleIPs` in `tailscale status --json`

## 3) Interaction Contract

- App starts by reading status once.
- Any action command:
  1. shows "running" feedback,
  2. executes command,
  3. displays success/failure,
  4. refreshes status.
- UI remains usable while not actively running a command.

## 4) Visual States

- `Idle`: normal render.
- `Busy`: command in progress, action keys temporarily disabled.
- `Error`: visible error message until next refresh/action.

## 5) Accessibility and Usability

- Keep text contrast high (terminal default-safe palette).
- Never require color alone to convey state.
- Keep key help visible in footer by default.
