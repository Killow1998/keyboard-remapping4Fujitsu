#!/usr/bin/env python3
"""Build the Key Layout amd64 Debian package."""

import argparse
import os
from pathlib import Path
import re
import shutil
import subprocess
import tempfile


ROOT = Path(__file__).resolve().parent.parent
DIST = ROOT / "dist"


def version():
    manifest = (ROOT / "Cargo.toml").read_text(encoding="utf-8")
    match = re.search(r'^version = "([^"]+)"$', manifest, re.MULTILINE)
    if not match:
        raise RuntimeError("Cannot read the package version from Cargo.toml")
    return match.group(1)


def write(path, content, mode=0o644):
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content, encoding="utf-8")
    path.chmod(mode)


def copy(source, destination, mode=0o644):
    destination.parent.mkdir(parents=True, exist_ok=True)
    shutil.copy2(source, destination)
    destination.chmod(mode)


def main():
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--no-build", action="store_true", help="package the existing release binary"
    )
    args = parser.parse_args()

    architecture = subprocess.run(
        ["dpkg", "--print-architecture"], check=True, text=True, capture_output=True
    ).stdout.strip()
    if architecture != "amd64":
        raise RuntimeError(f"This release package supports amd64, not {architecture}")
    if not args.no_build:
        subprocess.run(
            ["cargo", "build", "--release", "--locked"], cwd=ROOT, check=True
        )

    binary = ROOT / "target/release/key-layout"
    if not binary.is_file():
        raise RuntimeError("Missing target/release/key-layout")

    package_version = version()
    output = DIST / f"key-layout_{package_version}_{architecture}.deb"
    DIST.mkdir(exist_ok=True)

    with tempfile.TemporaryDirectory(prefix="key-layout-deb-") as temporary:
        tree = Path(temporary) / "key-layout"
        copy(binary, tree / "usr/bin/key-layout", 0o755)
        copy(ROOT / "README.md", tree / "usr/share/doc/key-layout/README.md")

        write(
            tree / "usr/share/applications/key-layout.desktop",
            "[Desktop Entry]\n"
            "Type=Application\n"
            "Name=Key Layout\n"
            "Comment=Customize keyboard keys\n"
            "Exec=/usr/bin/key-layout\n"
            "TryExec=/usr/bin/key-layout\n"
            "Icon=input-keyboard\n"
            "Terminal=false\n"
            "StartupNotify=true\n"
            "Categories=Settings;HardwareSettings;\n",
        )
        write(
            tree / "etc/xdg/autostart/key-layout.desktop",
            "[Desktop Entry]\n"
            "Type=Application\n"
            "Name=Key Layout mappings\n"
            "Comment=Keep saved keyboard mappings applied\n"
            "Exec=/usr/bin/key-layout --watch\n"
            "TryExec=/usr/bin/key-layout\n"
            "OnlyShowIn=XFCE;\n"
            "Terminal=false\n"
            "X-GNOME-Autostart-enabled=true\n",
        )
        write(
            tree / "usr/share/doc/key-layout/copyright",
            "Format: https://www.debian.org/doc/packaging-manuals/copyright-format/1.0/\n"
            "Upstream-Name: Key Layout\n"
            "Source: https://github.com/Killow1998/keyboard-remapping4Fujitsu\n\n"
            "Files: *\n"
            "Copyright: 2026 Killow1998\n"
            "License: All-rights-reserved\n"
            " This software is distributed without a license. No permission to copy,\n"
            " modify, or redistribute it is granted by default.\n",
        )

        installed_size = sum(
            path.stat().st_size for path in tree.rglob("*") if path.is_file()
        )
        write(
            tree / "DEBIAN/control",
            f"Package: key-layout\n"
            f"Version: {package_version}\n"
            f"Architecture: {architecture}\n"
            "Maintainer: Killow1998 <152495994+Killow1998@users.noreply.github.com>\n"
            "Installed-Size: " + str((installed_size + 1023) // 1024) + "\n"
            "Depends: libgtk-3-0t64 | libgtk-3-0, libx11-6, x11-xserver-utils, xfconf\n"
            "Section: utils\n"
            "Priority: optional\n"
            "Homepage: https://github.com/Killow1998/keyboard-remapping4Fujitsu\n"
            "Description: visual Japanese-keyboard remapper for XFCE on X11\n"
            " Remap unusual physical keys or use them to launch applications.\n"
            " Saved mappings are monitored and restored automatically after login.\n",
        )

        tree.chmod(0o755)
        for directory in (path for path in tree.rglob("*") if path.is_dir()):
            directory.chmod(0o755)

        environment = os.environ.copy()
        if "SOURCE_DATE_EPOCH" not in environment:
            environment["SOURCE_DATE_EPOCH"] = subprocess.run(
                ["git", "log", "-1", "--format=%ct"],
                cwd=ROOT,
                check=True,
                text=True,
                capture_output=True,
            ).stdout.strip()
        subprocess.run(
            ["dpkg-deb", "--root-owner-group", "--build", str(tree), str(output)],
            check=True,
            env=environment,
        )

    print(output)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
