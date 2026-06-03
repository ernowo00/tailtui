# tailtui

A small terminal UI for watching Tailscale status and running **up** / **down** using the **`tailscale` CLI only** (no control-plane HTTP API). Navigation follows a familiar **Vim-style** key layout.

Design goals follow **KISS** and **SOLID**: a thin TUI shell, clear module boundaries, and one place that knows how to talk to the CLI.

## Prerequisites

- [Rust](https://www.rust-lang.org/tools/install) toolchain (edition **2024**, recent stable).
- [Tailscale](https://tailscale.com/) installed with the `tailscale` binary on your `PATH`.
- A real interactive terminal (TUIs need a TTY; CI sandboxes often cannot run them).
- For **copy to clipboard** (`y` / `Y`): at least one clipboard helper from [Clipboard dependencies](#clipboard-dependencies) installed and reachable in your session.

### WSL2 (Tailscale on Windows)

If Tailscale runs on **Windows** but you use **tailtui** from a WSL2 distro (for example Arch) without a Linux `tailscale` package, use the install script. It symlinks the Windows `tailscale.exe` into `~/.local/bin`, updates your shell `PATH`, and runs `cargo install` for **tailtui**.

Prerequisites:

- [Tailscale](https://tailscale.com/) installed on Windows (default path: `C:\Program Files\Tailscale\tailscale.exe`).
- WSL can read `/mnt/c/...`.
- [Rust](https://www.rust-lang.org/tools/install) / `cargo` in WSL.

```bash
bash scripts/install-wsl.sh
# or: make install-wsl
```

Custom Windows binary path:

```bash
TAILSCALE_EXE='/mnt/c/Program Files/Tailscale/tailscale.exe' bash scripts/install-wsl.sh
```

Shell aliases (for example `alias ts='...tailscale.exe'`) do **not** apply to subprocesses; this script puts a real `tailscale` command on `PATH` instead. Clipboard copy still needs a Linux helper such as `xclip` (see [Clipboard dependencies](#clipboard-dependencies)); the script does not bridge clipboard to Windows.

## Build and run

```bash
cargo build
cargo run
```

Or use the Makefile:

```bash
make run
```

Release build:

```bash
cargo build --release
```

The binary ends up at `target/release/tailtui`.

## What it shows

- **Status** (read-only, no border): shown beside **Actions** in the top row—backend state, tailnet, device, IPs, exit node.
- **Actions**: in the top row next to status; trigger `tailscale up`, `tailscale down`, or refresh.
- **Machines**: full-width list of peers from `tailscale status --json`, sorted **online first**, then **MagicDNS / nickname** (fallback: hostname). Each row shows the tailnet name and, when it differs from the OS hostname, the hostname in parentheses. Offline rows are dimmed. When Tailscale is **stopped**, the list shows guidance instead of peer rows.
- **Feedback**: last command or clipboard result.

## Keybindings

| Key | Action |
| --- | --- |
| `h` / `l` | Move focus: **Machines** ↔ **Actions** (wraps; status is not focusable) |
| `j` / `k` | Move selection in focused panel (actions or machines) |
| `PgUp` / `PgDn` | Jump the machine selection by a page (only when **Machines** is focused) |
| `Enter` | Run the currently selected **action** |
| `u` | `tailscale up` |
| `d` | `tailscale down` |
| `r` | Refresh status (`tailscale status --json`) |
| `y` | Copy selected peer **IPv4** to the clipboard (when not stopped) |
| `Y` (Shift+y) | Copy **this device’s** first Tailscale **IPv4** (`Self.TailscaleIPs`) |
| `?` | Open help overlay |
| Any key (with help open) | Close help |
| `q` | Quit |

Header and feedback panels use **yellow** titles and borders; their body text is **white** to show they are not focus targets.

## Clipboard dependencies

Copy (`y` / `Y`) pipes text to an external helper. The app tries, in order:

1. **`wl-copy`** (Wayland, package `wl-clipboard` on Arch)
2. **`xclip -selection clipboard`** (X11)
3. **`xsel --clipboard --input`** (X11)

Install **one** of them; the first that works in your session wins. You need a display/clipboard bridge appropriate to your environment—for example **WSLg** or an X server on WSL2, or your desktop’s Wayland/X11 session on native Linux.

| Tool | Stack | When to use |
| --- | --- | --- |
| **`xclip`** | X11 | **Default recommendation for WSL2** (including Arch on WSL2): WSLg usually sets `DISPLAY=:0` and syncs the clipboard to Windows. |
| **`wl-copy`** | Wayland | Native Wayland desktops, or WSLg when your session is actually using Wayland (`WAYLAND_DISPLAY` set). |
| **`xsel`** | X11 | Fallback if you already use it; **`xclip`** is usually preferable. |

**Arch Linux (WSL2):**

```bash
sudo pacman -S xclip
```

Optional on Wayland-heavy setups:

```bash
sudo pacman -S wl-clipboard
```

On other distros, install the equivalent package for `xclip` and/or `wl-clipboard`. Without WSLg (or another X11/Wayland server), clipboard copy stays inside the Linux VM and will not reach the Windows host.

## Clipboard behavior

Copy uses short-lived subprocesses with a short wait so the UI should not block if a tool keeps a selection owner alive (common with `xclip` / `xsel`). If no supported helper is found or every attempt fails, the feedback panel explains the error.

## Specs and contracts

Product and engineering contracts live under `specs/`. Start with `specs/README.md`.

## Development

```bash
make help        # list Makefile targets
make test
make clippy      # warnings denied
```

## Something you should know

This tool shells out to whatever `tailscale` is on your `PATH`. Keep `tailscale` updated; JSON field shapes can vary slightly between versions, but parsing is written to be tolerant of optional fields.

## License

Licensed under the [MIT License](LICENSE).
