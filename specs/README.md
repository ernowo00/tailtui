# tailscale-tui Specs Index

This folder contains product and engineering contracts for `tailscale-tui`.

## Constitution

- `KISS`: keep implementation minimal, explicit, and easy to maintain.
- `SOLID`: isolate responsibilities, depend on abstractions, and keep modules composable.
- `No API`: only interact with Tailscale through the `tailscale` CLI binary.

## Documents

- `product-spec.md`: user-facing product behavior and UX scope.
- `architecture-contract.md`: module boundaries, dependencies, and SOLID rules.
- `cli-contract.md`: all command contracts to execute `tailscale` safely.
- `tui-contract.md`: keybindings, layouts, and navigation behavior.
- `acceptance-spec.md`: SLC completion criteria and test scenarios.

## Out of Scope (v1)

- Direct API integration with Tailscale.
- Editing ACL/policy files.
- Multi-account profile management.
- Node approval/admin workflows.
