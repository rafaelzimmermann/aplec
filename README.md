# aplec

A minimal Wayland clipboard history manager for Hyprland.

A background daemon watches the clipboard and persists the last 50 entries to disk. A layer-shell popup lets you browse history with the arrow keys and paste a previous entry back to the clipboard.

<img width="1914" height="1064" alt="image" src="https://github.com/user-attachments/assets/f88c462e-c847-4667-bd7b-4f780171eb9a" />


## Requirements

- Wayland compositor with `wlr-layer-shell` and `zwlr-data-control` support (Hyprland, Sway, etc.)
- [`wl-clipboard`](https://github.com/bugaevc/wl-clipboard) (`wl-copy`, `wl-paste`)

## Installation

Run the install script from the project root:

```sh
./scripts/install.sh          # installs to ~/.local/bin
./scripts/install.sh --system # installs to /usr/local/bin (requires sudo)
```

This builds a release binary, installs it, writes a default `~/.config/aplec/theme.conf`, and enables the systemd user service that runs the daemon.

The service is `WantedBy=default.target` and the daemon discovers the Wayland
socket itself (via `$WAYLAND_DISPLAY`, falling back to scanning
`$XDG_RUNTIME_DIR/wayland-*`). This keeps it working under any session
launcher — including Hyprland ≥ 0.56's `/usr/bin/start-hyprland`, which (unlike
an uwsm-managed session) never activates `graphical-session.target`.

To uninstall:

```sh
./scripts/install.sh --uninstall
```

## Hyprland setup

Add a keybind in `hyprland.conf`. Use the full path so the binary is found regardless of the compositor's PATH:

```conf
bind = $mainMod, V, exec, /home/you/.local/bin/aplec
```

Hyprland 0.56+ can also use the Lua config format:

```lua
hl.bind(mainMod .. " + V", hl.dsp.exec_cmd("~/.local/bin/aplec"))
```

## Usage

| Key         | Action                        |
|-------------|-------------------------------|
| `←` / `→`  | Browse clipboard history      |
| `Enter`     | Copy selected entry to clipboard |
| `Esc`       | Close without changing clipboard |

After pressing `Enter`, paste normally in the target application (`Ctrl+Shift+V` in terminals, `Ctrl+V` elsewhere).

## Configuration

Edit `~/.config/aplec/theme.conf`:

```conf
# placement: center | top-left | top-center | top-right
#            bottom-left | bottom-center | bottom-right
placement = center
margin    = 10      # gap in pixels from the nearest screen edge

background = #1e1e2eee
text       = #cdd6f4
text_dim   = #6c7086
accent     = #cba6f7
selected_bg = #313244
border     = #45475a
```

All keys are optional — omitted keys fall back to the defaults shown above. Colours are `#RRGGBB` or `#RRGGBBAA`.

## How it works

- **Daemon** — polls `wl-paste` every 500 ms, deduplicates entries, and appends new ones to `~/.local/share/aplec/history.json` (capped at 50).
- **Popup** — reads `history.json` on launch, renders a full-screen overlay (transparent outside the card), and exits after copying the selected entry back via `wl-copy`.
