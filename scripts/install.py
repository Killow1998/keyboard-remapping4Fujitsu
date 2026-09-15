#!/usr/bin/env python3
"""Install or uninstall Key Layout for the current user."""
import argparse
import os
from pathlib import Path
import shutil
import subprocess
import sys

ROOT = Path(__file__).resolve().parent.parent


def paths():
    home = Path.home()
    data = Path(os.environ.get("XDG_DATA_HOME", home / ".local/share"))
    config = Path(os.environ.get("XDG_CONFIG_HOME", home / ".config"))
    return (
        home / ".local/bin/key-layout",
        data / "applications/key-layout.desktop",
        config / "autostart/key-layout.desktop",
        config / "key-layout/rules.tsv",
    )


def desktop_argument(value):
    # Exec quoting includes Desktop Entry string escaping and literal percent signs.
    value = value.replace("\\", "\\\\\\\\")
    for char in ('"', '`', '$'):
        value = value.replace(char, "\\\\" + char)
    return '"' + value.replace('%', '%%') + '"'


def install(no_build):
    if not no_build:
        subprocess.run(["cargo", "build", "--release", "--locked"], cwd=ROOT, check=True)
    source = ROOT / "target/release/key-layout"
    if not source.is_file():
        raise RuntimeError("Build first with cargo build --release --locked.")
    binary, launcher, _, _ = paths()
    binary.parent.mkdir(parents=True, exist_ok=True)
    temporary = binary.with_suffix(".new")
    shutil.copy2(source, temporary)
    temporary.chmod(0o755)
    temporary.replace(binary)
    launcher.parent.mkdir(parents=True, exist_ok=True)
    launcher.write_text(
        "[Desktop Entry]\n"
        "Type=Application\n"
        "Name=Key Layout\n"
        "Comment=Customize keyboard keys\n"
        f"Exec={desktop_argument(str(binary))}\n"
        "Icon=input-keyboard\n"
        "Terminal=false\n"
        "Categories=Settings;HardwareSettings;\n",
        encoding="utf-8",
    )
    print(f"Installed: {binary}\nOpen Key Layout from the application menu.")


def uninstall():
    binary, launcher, autostart, rules = paths()
    if rules.exists() and rules.read_text(encoding="utf-8").strip():
        if not binary.is_file():
            raise RuntimeError("Reinstall the app, then restore your mappings before uninstalling.")
        # Stop on failure: active mappings must remain recoverable.
        subprocess.run([str(binary), "--restore"], check=True)
    for path in (autostart, launcher, binary):
        path.unlink(missing_ok=True)
    print("Uninstalled. The settings directory was retained.")


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    options = parser.add_mutually_exclusive_group()
    options.add_argument("--no-build", action="store_true", help="install the existing release build")
    options.add_argument("--uninstall", action="store_true", help="restore mappings and remove the app")
    args = parser.parse_args()
    try:
        uninstall() if args.uninstall else install(args.no_build)
    except (OSError, RuntimeError, subprocess.CalledProcessError) as error:
        print(f"Error: {error}", file=sys.stderr)
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
