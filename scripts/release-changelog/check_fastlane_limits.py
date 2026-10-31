#!/usr/bin/env python3
"""Check fastlane store-metadata fields against their per-store character limits.

Covers EVERY length-limited fastlane field, not just changelogs — the single guard for the whole
store-metadata surface (Google Play + App Store). Used two ways:
  - CI (check-fastlane-metadata.yml): pass the PR's changed metadata files as args.
  - In-generator (release-store-notes.yml): run with --all before opening the store-notes PR.

Character count: trailing whitespace/newlines are stripped before counting (the stores trim them),
so a field whose content is exactly at the limit passes even with a trailing newline. This matches
translate.py's auto-fit, which measures the content without its trailing "\\n".

Limits are re-verified against current Play/App Store docs at implementation time (they change).

Usage:
    check_fastlane_limits.py [--root DIR] [--all] [FILE ...]
Exit status 1 if any checked field exceeds its limit; 0 otherwise.
"""
import argparse
import os
import re
import sys

# (regex on the path relative to repo root, limit, human label, unit)
# unit "bytes" for fields the store measures in bytes (Apple keywords), else "chars".
RULES = [
    # --- Google Play (fastlane/metadata/android/<locale>/...) ---
    (r"fastlane/metadata/android/[^/]+/title\.txt$",                 30,   "Play title",            "chars"),
    (r"fastlane/metadata/android/[^/]+/short_description\.txt$",     80,   "Play short description","chars"),
    (r"fastlane/metadata/android/[^/]+/full_description\.txt$",      4000, "Play full description", "chars"),
    (r"fastlane/metadata/android/[^/]+/changelogs/[^/]+\.txt$",      500,  "Play changelog",        "chars"),
    # --- Apple App Store (nym-vpn-apple/fastlane/metadata/<locale>/...) ---
    (r"nym-vpn-apple/fastlane/metadata/[^/]+/name\.txt$",            30,   "App Store name",        "chars"),
    (r"nym-vpn-apple/fastlane/metadata/[^/]+/subtitle\.txt$",        30,   "App Store subtitle",    "chars"),
    (r"nym-vpn-apple/fastlane/metadata/[^/]+/promotional_text\.txt$",170,  "App Store promo text",  "chars"),
    (r"nym-vpn-apple/fastlane/metadata/[^/]+/keywords\.txt$",        100,  "App Store keywords",    "bytes"),
    (r"nym-vpn-apple/fastlane/metadata/[^/]+/description\.txt$",     4000, "App Store description", "chars"),
    (r"nym-vpn-apple/fastlane/metadata/[^/]+/release_notes\.txt$",   4000, "App Store release notes","chars"),
]
COMPILED = [(re.compile(p), lim, label, unit) for p, lim, label, unit in RULES]

# Roots walked by --all
SCAN_DIRS = ["fastlane/metadata/android", "nym-vpn-apple/fastlane/metadata"]


def rule_for(rel_path):
    """Return (limit, label, unit) for the first rule matching this repo-relative path, else None."""
    rel = rel_path.replace(os.sep, "/")
    for pat, lim, label, unit in COMPILED:
        if pat.search(rel):
            return lim, label, unit
    return None


def field_len(path, unit="chars"):
    """Length as the store counts it, trailing whitespace stripped: characters, or UTF-8 bytes for
    byte-limited fields (Apple keywords), where a multi-byte locale can fit fewer characters."""
    with open(path, encoding="utf-8") as f:
        s = f.read().rstrip()
    return len(s.encode("utf-8")) if unit == "bytes" else len(s)


def iter_all(root):
    for base in SCAN_DIRS:
        for dirpath, _dirs, files in os.walk(os.path.join(root, base)):
            for name in files:
                if name.endswith(".txt"):
                    yield os.path.join(dirpath, name)


def main():
    ap = argparse.ArgumentParser(description=__doc__)
    ap.add_argument("files", nargs="*", help="metadata files to check (repo-relative or absolute)")
    ap.add_argument("--root", default=os.environ.get("GITHUB_WORKSPACE", os.getcwd()),
                    help="repo root (default: $GITHUB_WORKSPACE or cwd)")
    ap.add_argument("--all", action="store_true", help="scan the whole fastlane metadata tree")
    args = ap.parse_args()

    root = os.path.abspath(args.root)
    paths = list(iter_all(root)) if args.all else [
        p if os.path.isabs(p) else os.path.join(root, p) for p in args.files]

    checked = failed = 0
    for path in paths:
        rel = os.path.relpath(path, root)
        rule = rule_for(rel)
        if rule is None:
            continue  # not a length-limited field — ignore (e.g. video.txt, URLs)
        if not os.path.isfile(path):
            continue  # deleted in the diff
        limit, label, unit = rule
        n = field_len(path, unit)
        checked += 1
        status = "OK" if n <= limit else "FAIL"
        print(f"  [{status}] {label:24} {n:5}/{limit:<5} {unit:5} {rel}")
        if n > limit:
            print(f"::error file={rel}::{label} exceeds the store limit of {limit} "
                  f"{unit} ({n}).")
            failed += 1

    if not checked:
        print("No length-limited fastlane fields to check.")
    print(f"checked {checked} field(s), {failed} over limit")
    sys.exit(1 if failed else 0)


if __name__ == "__main__":
    main()
