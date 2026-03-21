#!/usr/bin/env bash
# Install aplec
#
# Options:
#   --user        install to ~/.local/bin instead of /usr/local/bin
#   --system      install to /usr/local/bin
#   --uninstall   remove installed files
#   --yes         assume yes for all prompts (non-interactive)
#   --no          assume no for all prompts (non-interactive)

set -euo pipefail

# ── Argument parsing ──────────────────────────────────────────────────────────

SYSTEM=false
UNINSTALL=false
YES=false
NO=false

for arg in "$@"; do
    case "$arg" in
        --user)      SYSTEM=false ;;
        --system)    SYSTEM=true ;;
        --uninstall) UNINSTALL=true ;;
        --yes|-y)    YES=true ;;
        --no|-n)     NO=true ;;
        --help|-h)
            sed -n '2,9p' "$0" | sed 's/^# \?//'
            exit 0
            ;;
        *)
            echo "Unknown option: $arg"
            echo "Run with --help to see available options."
            exit 1
            ;;
    esac
done

# ── Paths ─────────────────────────────────────────────────────────────────────

if $SYSTEM; then
    BIN_DIR="/usr/local/bin"
else
    BIN_DIR="${HOME}/.local/bin"
fi

BINARY="$BIN_DIR/aplec"
CONFIG_DIR="${HOME}/.config/aplec"
THEME_FILE="$CONFIG_DIR/theme.conf"

# ── Privilege helpers ─────────────────────────────────────────────────────────

if $SYSTEM && [[ $EUID -ne 0 ]]; then
    PRIV="sudo"
    echo "sudo access is required for system-wide install."
    sudo -v
    ( while true; do sudo -n true; sleep 50; done ) &
    SUDO_KEEPALIVE_PID=$!
    trap 'kill "$SUDO_KEEPALIVE_PID" 2>/dev/null' EXIT
else
    PRIV=""
fi

priv_mkdir()   { $PRIV mkdir -p "$@"; }
priv_install() { $PRIV install "$@"; }
priv_rm()      { $PRIV rm "$@"; }
priv_rmdir()   { $PRIV rmdir "$@" 2>/dev/null || true; }

confirm() {
    if $YES; then return 0; fi
    if $NO;  then return 1; fi
    local reply
    read -r -n 1 -p "$1 [y/N] " reply
    echo ""
    [[ "${reply,,}" == "y" ]]
}

# ── Uninstall ─────────────────────────────────────────────────────────────────

if $UNINSTALL; then
    echo "Uninstalling aplec…"
    if systemctl --user is-active --quiet aplec 2>/dev/null; then
        systemctl --user disable --now aplec
    fi
    rm -f "${HOME}/.config/systemd/user/aplec.service"
    systemctl --user daemon-reload 2>/dev/null || true
    priv_rm -f "$BINARY"
    rm -rf "$CONFIG_DIR"
    echo "Done."
    exit 0
fi

# ── Source check ──────────────────────────────────────────────────────────────

if [[ ! -f Cargo.toml ]]; then
    echo "Error: run this script from the aplec project root." >&2
    exit 1
fi

# ── Build ──────────────────────────────────────────────────────────────────────

echo "Building aplec (release)…"
cargo build --release

# ── Install binary ────────────────────────────────────────────────────────────

echo "Installing to $BIN_DIR…"
priv_mkdir "$BIN_DIR"
priv_install -m 755 target/release/aplec "$BINARY"

# ── Config ────────────────────────────────────────────────────────────────────

mkdir -p "$CONFIG_DIR"

if [[ ! -f "$THEME_FILE" ]]; then
    cat > "$THEME_FILE" <<'EOF'
# aplec theme configuration
# placement: center, top-right, top-left, top-center, bottom-right, bottom-left, bottom-center
placement = center
margin = 10
# background = #1e1e2eee
# text = #cdd6f4
# text_dim = #6c7086
# accent = #cba6f7
# selected_bg = #313244
# border = #45475a
EOF
    echo "Default theme config installed to $THEME_FILE."
else
    echo "Keeping existing theme config."
fi

# ── Systemd user service ──────────────────────────────────────────────────────

SYSTEMD_DIR="${HOME}/.config/systemd/user"
SERVICE_FILE="$SYSTEMD_DIR/aplec.service"

mkdir -p "$SYSTEMD_DIR"
cat > "$SERVICE_FILE" <<EOF
[Unit]
Description=aplec clipboard history daemon
After=graphical-session.target

[Service]
ExecStart=$BINARY daemon
Restart=on-failure

[Install]
WantedBy=graphical-session.target
EOF

systemctl --user daemon-reload
systemctl --user enable --now aplec
echo "Daemon enabled and started via systemd."

# ── Summary ───────────────────────────────────────────────────────────────────

echo ""
echo "Installed:  $BINARY"
echo "Service:    $SERVICE_FILE"
echo "Theme:      $THEME_FILE"
echo ""

if ! command -v aplec &>/dev/null 2>&1; then
    echo "Note: $BIN_DIR is not on your PATH."
    echo "Add it to your shell profile:"
    echo "  export PATH=\"\$PATH:$BIN_DIR\""
    echo ""
fi

echo "Open the popup: bind 'aplec' to a hotkey in your compositor"
echo "  (e.g. Hyprland: bind = \$mod, V, exec, aplec)"
