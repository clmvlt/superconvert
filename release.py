#!/usr/bin/env python3
"""
Release script for Convertor.
Reads the current version, creates a git tag, pushes it,
and triggers the GitHub Actions release workflow.

Usage:
    python release.py          # Release with current version from tauri.conf.json
    python release.py --bump patch  # Bump patch version (0.0.1 -> 0.0.2) then release
    python release.py --bump minor  # Bump minor version (0.0.1 -> 0.1.0) then release
    python release.py --bump major  # Bump major version (0.0.1 -> 1.0.0) then release

Requirements:
    - gh CLI installed and authenticated (https://cli.github.com/)
    - git configured with push access to origin
"""

import json
import subprocess
import sys
import argparse
from pathlib import Path

ROOT = Path(__file__).resolve().parent
TAURI_CONF = ROOT / "src-tauri" / "tauri.conf.json"
PACKAGE_JSON = ROOT / "package.json"
CARGO_TOML = ROOT / "src-tauri" / "Cargo.toml"


def run(cmd: list[str], check: bool = True, capture: bool = False) -> subprocess.CompletedProcess:
    print(f"  > {' '.join(cmd)}")
    return subprocess.run(cmd, check=check, capture_output=capture, text=True, cwd=ROOT)


def get_version() -> str:
    conf = json.loads(TAURI_CONF.read_text())
    return conf["version"]


def bump_version(version: str, part: str) -> str:
    major, minor, patch = (int(x) for x in version.split("."))
    if part == "major":
        major += 1
        minor = 0
        patch = 0
    elif part == "minor":
        minor += 1
        patch = 0
    elif part == "patch":
        patch += 1
    return f"{major}.{minor}.{patch}"


def set_version(new_version: str) -> None:
    # tauri.conf.json
    conf = json.loads(TAURI_CONF.read_text())
    conf["version"] = new_version
    TAURI_CONF.write_text(json.dumps(conf, indent=2) + "\n")

    # package.json
    pkg = json.loads(PACKAGE_JSON.read_text())
    pkg["version"] = new_version
    PACKAGE_JSON.write_text(json.dumps(pkg, indent=2) + "\n")

    # Cargo.toml
    cargo = CARGO_TOML.read_text()
    lines = cargo.splitlines()
    new_lines = []
    in_package = False
    version_replaced = False
    for line in lines:
        if line.strip() == "[package]":
            in_package = True
        elif line.strip().startswith("[") and line.strip() != "[package]":
            in_package = False
        if in_package and not version_replaced and line.strip().startswith("version"):
            new_lines.append(f'version = "{new_version}"')
            version_replaced = True
        else:
            new_lines.append(line)
    CARGO_TOML.write_text("\n".join(new_lines) + "\n")


def check_prerequisites() -> None:
    # Check gh CLI
    result = run(["gh", "--version"], check=False, capture=True)
    if result.returncode != 0:
        print("Error: gh CLI is not installed. Install it from https://cli.github.com/")
        sys.exit(1)

    # Check gh auth
    result = run(["gh", "auth", "status"], check=False, capture=True)
    if result.returncode != 0:
        print("Error: gh CLI is not authenticated. Run: gh auth login")
        sys.exit(1)

    # Check clean working tree (allow untracked)
    result = run(["git", "diff", "--quiet"], check=False, capture=True)
    staged = run(["git", "diff", "--cached", "--quiet"], check=False, capture=True)
    if result.returncode != 0 or staged.returncode != 0:
        print("Error: Working tree has uncommitted changes. Commit or stash them first.")
        sys.exit(1)


def main() -> None:
    parser = argparse.ArgumentParser(description="Release Convertor")
    parser.add_argument(
        "--bump",
        choices=["major", "minor", "patch"],
        help="Bump version before releasing",
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Show what would be done without actually doing it",
    )
    args = parser.parse_args()

    print("=== Convertor Release Script ===\n")

    # 1. Check prerequisites
    print("[1/5] Checking prerequisites...")
    if not args.dry_run:
        check_prerequisites()
    print("  OK\n")

    # 2. Determine version
    current = get_version()
    print(f"[2/5] Current version: {current}")

    if args.bump:
        version = bump_version(current, args.bump)
        print(f"  Bumping {args.bump}: {current} -> {version}")
        if not args.dry_run:
            set_version(version)
            run(["git", "add", str(TAURI_CONF), str(PACKAGE_JSON), str(CARGO_TOML)])
            run(["git", "commit", "-m", f"chore: bump version to {version}"])
    else:
        version = current

    tag = f"v{version}"
    print(f"  Release tag: {tag}\n")

    # 3. Check if tag already exists
    print("[3/5] Checking tag...")
    result = run(["git", "tag", "-l", tag], capture=True)
    if tag in result.stdout:
        print(f"  Error: Tag {tag} already exists. Bump the version or delete the tag.")
        sys.exit(1)
    print("  Tag is available\n")

    # 4. Create and push tag
    print(f"[4/5] Creating tag {tag}...")
    if not args.dry_run:
        run(["git", "tag", tag])
        run(["git", "push", "origin", "main"])
        run(["git", "push", "origin", tag])
    else:
        print("  (dry-run) Would create and push tag\n")

    # 5. Trigger GitHub Actions workflow
    print("[5/5] Triggering release workflow...")
    if not args.dry_run:
        run(["gh", "workflow", "run", "release.yml", "-f", f"version={tag}"])
        print(f"\n  Release workflow triggered for {tag}!")
        print("  Monitor progress: https://github.com/clmvlt/convertor/actions")
    else:
        print(f"  (dry-run) Would trigger workflow for {tag}")

    print(f"\n=== Done! Convertor {tag} release in progress ===")


if __name__ == "__main__":
    main()
