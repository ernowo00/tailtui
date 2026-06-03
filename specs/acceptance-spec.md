# Acceptance Spec: SLC v1

## Definition of Done

Project is "simple, lovable, complete" when all checks below pass.

## 1) Simple

- Codebase respects defined module boundaries.
- New contributor can identify where to add:
  - UI keybinding
  - New status field parser
  - New action command
- No API client code exists.

## 2) Lovable

- Startup-to-first-status is fast and clear.
- Vim navigation works consistently (`h j k l`).
- User gets immediate feedback after `u` / `d`.
- Error messages are actionable (what failed + what to retry).

## 3) Complete

- Read and display local Tailscale status.
- Trigger `tailscale up`.
- Trigger `tailscale down`.
- Refresh after each action.
- Quit cleanly without terminal corruption.

## 4) Test Scenarios

## Functional

1. `tailscale` installed and daemon running:
   - App shows connected/disconnected state.
2. Trigger `u`:
   - Command executes, success message shown, status updates.
3. Trigger `d`:
   - Command executes, success message shown, status updates.
4. Trigger `r`:
   - Status panel refreshes.

## Failure handling

1. `tailscale` missing:
   - App shows `CliNotFound` message.
2. Permission issue:
   - App shows command failure message and remains usable.
3. Timeout:
   - App shows timeout message and allows retry.
4. Invalid/unexpected output:
   - App shows parse error, no crash.

## 5) Non-Goals Validation

- Confirm no direct HTTP/API calls in code.
- Confirm no login/admin flow features shipped in v1.
