# Key Layout

<p align="center"><img src="assets/key-layout.svg" alt="Key Layout icon" width="128"></p>

**Experimental — tested only on a Fujitsu LIFEBOOK U9310 laptop with a Japanese keyboard, running XFCE on X11. Other laptops and desktop environments have not been tested.**

Key Layout is a small visual remapper for the unusual keys on a Japanese Fujitsu U9310 keyboard. It can make a physical key act like another key or launch a desktop application. The keyboard view shows the original key and its saved action.

## Requirements

- XFCE running on X11
- Rust 1.93 or newer to build
- `xmodmap` and `xfconf-query`

## Install

### Debian package

Download the amd64 `.deb` from the GitHub Release, then install it with:

```sh
sudo apt install ./key-layout_0.1.2_amd64.deb
```

The package installs an XFCE autostart entry. The mapping watcher starts automatically at the next XFCE login and runs as that desktop user. Open **Key Layout** from the application menu to create mappings.

### From source

```sh
python3 scripts/install.py
```

Open **Key Layout** from the application menu. Select a key, choose its new action, and press **Apply**. Applying the first rule also creates an XFCE autostart entry.

Saved mappings are watched after login and restored when X11, a keyboard-layout switch, or resume resets them. The watcher reads the current rules on every check, uses a direct X11 query while idle, and only runs `xmodmap` when a saved mapping has drifted. **Reapply** remains available as a manual recovery action.

Settings are stored in:

```text
~/.config/key-layout/rules.tsv
~/.config/autostart/key-layout.desktop
```

## Remove

Restore all mappings in the app before removing a Debian installation, then run:

```sh
sudo apt remove key-layout
```

For a source installation, run:

```sh
python3 scripts/install.py --uninstall
```

Uninstall restores the original key mappings before removing the application and autostart entry. The settings file is retained.

## Scope and limitations

- X11 keycodes are hardware and layout specific. Do not copy `rules.tsv` to another keyboard without checking its physical keycodes.
- Modifier and Fn keys are intentionally read-only.
- Application shortcuts use XFCE keyboard shortcuts.
- This repository has no license; no permission to copy, modify, or redistribute is granted by default.
