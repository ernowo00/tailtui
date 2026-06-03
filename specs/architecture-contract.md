# Architecture Contract: tailscale-tui

## 1) Design Rules (KISS + SOLID)

- Single responsibility per module.
- UI layer does not execute shell commands directly.
- Domain layer defines interfaces and state models only.
- Infra layer implements CLI execution details.
- All command strings are centralized and tested.
- Keep dependencies minimal.

## 2) Module Boundaries

## `app` (composition)

- Wire concrete implementations to interfaces.
- Own startup, shutdown, and main event loop orchestration.

## `domain`

- Data models:
  - `TailscaleStatus`
  - `ActionResult`
  - `AppState`
- Traits (abstractions):
  - `StatusReader`
  - `TailscaleController`

## `infra::cli`

- Implements domain traits via `std::process::Command`.
- Responsible for:
  - Command execution
  - Timeout handling
  - Exit code/stdout/stderr mapping to domain errors
  - Parsing CLI output into domain model

## `ui::tui`

- Handles terminal rendering and input mapping.
- Converts key events into domain-intent actions.
- Renders current `AppState` and transient feedback messages.

## 3) Dependency Direction

- `ui` -> depends on domain abstractions only.
- `app` -> depends on domain abstractions and infra implementations.
- `domain` -> depends on nothing outside std/core.
- `infra` -> depends on domain traits/models.

No reverse dependencies allowed.

## 4) Error Contract

All recoverable errors must map to user-visible message + internal reason:

- `CliNotFound`
- `CommandFailed`
- `ParseFailed`
- `Timeout`
- `PermissionDenied`

UI must remain responsive after any recoverable error.

## 5) Initial File Layout Contract

```text
src/
  main.rs
  app/
    mod.rs
    run.rs
  domain/
    mod.rs
    model.rs
    ports.rs
    error.rs
  infra/
    mod.rs
    cli/
      mod.rs
      command_runner.rs
      parser.rs
  ui/
    mod.rs
    tui/
      mod.rs
      event.rs
      render.rs
```
