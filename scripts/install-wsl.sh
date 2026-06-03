#!/usr/bin/env bash
# WSL2 setup: symlink Windows tailscale.exe onto PATH and install tailtui.
set -euo pipefail

PATH_MARKER='# tailscale-tui WSL install'
DEFAULT_TAILSCALE_EXE='/mnt/c/Program Files/Tailscale/tailscale.exe'

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

TAILSCALE_EXE="${TAILSCALE_EXE:-}"
TOUCHED_FILES=()

usage() {
    cat <<'EOF'
Usage: bash scripts/install-wsl.sh [OPTIONS] [TAILSCALE_EXE]

Install tailtui and put the Windows Tailscale CLI on your WSL PATH.

Options:
  -h, --help    Show this help and exit

Environment:
  TAILSCALE_EXE   Path to tailscale.exe (overrides default and positional arg)

Default tailscale.exe:
  /mnt/c/Program Files/Tailscale/tailscale.exe

Examples:
  bash scripts/install-wsl.sh
  TAILSCALE_EXE='/mnt/c/Program Files/Tailscale/tailscale.exe' bash scripts/install-wsl.sh
  bash scripts/install-wsl.sh '/mnt/c/Program Files/Tailscale/tailscale.exe'
EOF
}

log() {
    printf '%s\n' "$*"
}

warn() {
    printf 'warning: %s\n' "$*" >&2
}

die() {
    printf 'error: %s\n' "$*" >&2
    exit 1
}

parse_args() {
    while [[ $# -gt 0 ]]; do
        case "$1" in
            -h | --help)
                usage
                exit 0
                ;;
            --)
                shift
                break
                ;;
            -*)
                die "unknown option: $1 (try --help)"
                ;;
            *)
                if [[ -z "${TAILSCALE_EXE}" ]]; then
                    TAILSCALE_EXE="$1"
                else
                    die "unexpected argument: $1"
                fi
                shift
                return
                ;;
        esac
        shift
    done
}

check_wsl() {
    if [[ -r /proc/version ]] && grep -qiE 'microsoft|wsl' /proc/version; then
        return 0
    fi
    warn 'this does not look like WSL; continuing anyway'
}

resolve_tailscale_exe() {
    if [[ -z "${TAILSCALE_EXE}" ]]; then
        TAILSCALE_EXE="${DEFAULT_TAILSCALE_EXE}"
    fi
    if [[ ! -f "${TAILSCALE_EXE}" ]]; then
        die "tailscale.exe not found at: ${TAILSCALE_EXE}
Install Tailscale on Windows, or pass the path:
  bash scripts/install-wsl.sh '/path/to/tailscale.exe'"
    fi
}

require_cargo() {
    if ! command -v cargo >/dev/null 2>&1; then
        die 'cargo not found on PATH; install Rust from https://rustup.rs/'
    fi
}

install_tailscale_shim() {
    local link="${HOME}/.local/bin/tailscale"
    mkdir -p "${HOME}/.local/bin"
    ln -sf "${TAILSCALE_EXE}" "${link}"
    log "linked ${link} -> ${TAILSCALE_EXE}"
}

append_path_block() {
    local rc="$1"
    if [[ ! -f "${rc}" ]]; then
        return 0
    fi
    if grep -qF "${PATH_MARKER}" "${rc}" 2>/dev/null; then
        log "PATH block already present in ${rc}"
        return 0
    fi
    {
        printf '\n%s\n' "${PATH_MARKER}"
        printf 'export PATH="$HOME/.local/bin:$PATH"\n'
        printf 'export PATH="$HOME/.cargo/bin:$PATH"\n'
    } >>"${rc}"
    TOUCHED_FILES+=("${rc}")
    log "updated ${rc}"
}

update_shell_path() {
    append_path_block "${HOME}/.bashrc"
    append_path_block "${HOME}/.zshrc"
    append_path_block "${HOME}/.profile"
}

install_tailtui() {
    log "installing tailtui from ${REPO_ROOT}..."
    if ! (cd "${REPO_ROOT}" && cargo install --path . --locked); then
        die 'cargo install failed; try: rustup update stable'
    fi
}

export_path_for_verify() {
    export PATH="${HOME}/.local/bin:${HOME}/.cargo/bin:${PATH}"
}

verify_install() {
    local failed=0

    if ! command -v tailscale >/dev/null 2>&1; then
        warn 'tailscale not found on PATH after install'
        failed=1
    elif ! tailscale version >/dev/null 2>&1; then
        warn 'tailscale version failed (is the Windows Tailscale service running?)'
        failed=1
    else
        log "tailscale: $(command -v tailscale)"
        tailscale version | head -n1
    fi

    if ! command -v tailtui >/dev/null 2>&1; then
        warn 'tailtui not found on PATH after install'
        failed=1
    else
        log "tailtui: $(command -v tailtui)"
    fi

    if [[ "${failed}" -ne 0 ]]; then
        die 'verification failed; open a new shell and ensure ~/.local/bin and ~/.cargo/bin are on PATH'
    fi
}

print_next_steps() {
    log ''
    log 'Done.'
    if [[ ${#TOUCHED_FILES[@]} -gt 0 ]]; then
        log 'Reload your shell config or open a new terminal:'
        for rc in "${TOUCHED_FILES[@]}"; do
            log "  source ${rc}"
        done
    else
        log 'Open a new terminal if tailscale or tailtui are not found yet.'
    fi
    log 'Run: tailtui'
}

main() {
    parse_args "$@"
    check_wsl
    resolve_tailscale_exe
    require_cargo
    install_tailscale_shim
    update_shell_path
    install_tailtui
    export_path_for_verify
    verify_install
    print_next_steps
}

main "$@"
