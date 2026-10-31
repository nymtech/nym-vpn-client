#!/usr/bin/env python3
"""(b) Author consumer app-store copy by DERIVING from the component CHANGELOG `[Unreleased]`.

This is the chain step: instead of querying PRs again, it reads the `[Unreleased]` bullets that
author_changelog.py just wrote, so the store copy can't contradict the changelog and reuses its
user-facing/ranking/routing decisions. The store stage still REWRITES for consumer voice + length.

    Android store copy  <-  nym-vpn-android + nym-vpn-core  [Unreleased]
    Apple store copy    <-  nym-vpn-apple   + nym-vpn-core  [Unreleased]

Writes one bullet per line to out/android.en.txt and out/apple.en.txt (translate.py renders the
per-locale fastlane files from these).

Env: ANTHROPIC_API_KEY (required unless --no-llm), ANTHROPIC_MODEL (optional).
Flags: --dry-run (print, write nothing) · --no-llm (pass the raw changelog bullets through, for a
plumbing smoke test only — not real store copy).
"""
import json
import os
import re
import sys
import urllib.request

ROOT = os.environ.get("GITHUB_WORKSPACE", os.getcwd())
MODEL = os.environ.get("ANTHROPIC_MODEL", "claude-opus-4-8")
HERE = os.path.dirname(os.path.abspath(__file__))
OUT = os.path.join(HERE, "out")


def load_style():
    """The committed house style/terminology, loaded into the store-copy prompt."""
    try:
        return open(os.path.join(HERE, "style.md"), encoding="utf-8").read().strip()
    except OSError:
        return ""


STYLE = load_style()

STORE_SOURCES = {
    "android": ["nym-vpn-android/CHANGELOG.md", "nym-vpn-core/CHANGELOG.md"],
    "apple":   ["nym-vpn-apple/CHANGELOG.md", "nym-vpn-core/CHANGELOG.md"],
}

SYSTEM = """You write app-store "What's new" release notes for the NymVPN apps, from the \
developer changelog entries of the changes going into this release.

Rules:
- Input is the technical changelog bullets. Your job is to SELECT and REWRITE the user-facing \
ones as app-store copy — not to copy them. Drop anything internal (engine refactors, deps, CI, \
build, test).
- Rank by user impact: new features first, then notable fixes, then minor improvements.
- Plain, benefit-oriented language. No developer wording, no PR links, no "[Android]" prefixes, \
no markdown, no bullet characters.
- Android (Google Play): keep very tight — aim <= 320 characters total, about 5 short lines — it \
must survive ~60% translation expansion under the 500-char limit.
- Apple (App Store): more room (about 7 lines max), still concise.
- Return ONLY JSON, no prose or code fences: {"android": ["line", ...], "apple": ["line", ...]}"""


def _section_body(text, header_regex):
    m = re.search(rf"{header_regex}\n(.*?)(?=\n## \[|\Z)", text, re.S)
    return m.group(1) if m else ""


def _bullets(body):
    out = []
    for line in body.splitlines():
        line = line.strip()
        if line.startswith("- "):
            out.append(re.sub(r"\s*\(https?://\S+\)\s*$", "", line[2:]).strip())
    return out


def unreleased_bullets(path, allow_fallback):
    """Bullets for the release being prepared.

    Reads `[Unreleased]` first. If it is empty, the fallback is only safe at the FINAL release, where
    `add-changelog-release` has just moved this release's bullets into the newest `## [version] - date`
    section — so `allow_fallback` must be passed only then. At a beta (no stamp) there is no section
    for the current release, so falling back would grab the PREVIOUS release's copy; betas pass
    `allow_fallback=False` and get an empty result instead.

    The fallback regex matches the DATED header (`## [ver] - date`). The old `## \\[\\d[^\\]]*\\]`
    required `]` immediately before the newline, so it never matched a dated section and silently ran
    on to an ancient UNDATED one (e.g. `## [2.8.0]`) — the bug this replaces. Empty results are loud,
    never silent."""
    with open(os.path.join(ROOT, path), encoding="utf-8") as f:
        text = f.read()
    bullets = _bullets(_section_body(text, r"## \[Unreleased\]"))
    if bullets:
        return bullets
    if not allow_fallback:
        return []
    fb = _bullets(_section_body(text, r"## \[\d[^\]]*\][^\n]*"))  # newest dated section
    if fb:
        print(f"::notice::{path}: [Unreleased] empty; using the latest stamped release section")
    else:
        print(f"::warning::{path}: no [Unreleased] bullets and no stamped release section — "
              f"store copy for this component will be empty")
    return fb


def gather(allow_fallback):
    return {store: sorted({b for p in paths for b in unreleased_bullets(p, allow_fallback)})
            for store, paths in STORE_SOURCES.items()}


def call_anthropic(sources):
    body = {
        "model": MODEL, "max_tokens": 1500,
        "system": SYSTEM + (f"\n\n{STYLE}" if STYLE else ""),
        "messages": [{"role": "user", "content":
            "Changelog entries for this release:\n"
            f"Android (android + core):\n- " + "\n- ".join(sources["android"]) +
            "\n\nApple (apple + core):\n- " + "\n- ".join(sources["apple"])}],
    }
    headers = {"x-api-key": os.environ["ANTHROPIC_API_KEY"],
               "anthropic-version": "2023-06-01", "content-type": "application/json"}
    ws = os.environ.get("ANTHROPIC_WORKSPACE_ID")  # needed only for org-scoped keys
    if ws:
        headers["anthropic-workspace-id"] = ws
    req = urllib.request.Request(
        "https://api.anthropic.com/v1/messages",
        data=json.dumps(body).encode("utf-8"), headers=headers)
    with urllib.request.urlopen(req, timeout=120) as resp:
        data = json.load(resp)
    text = "".join(b.get("text", "") for b in data["content"])
    s, e = text.find("{"), text.rfind("}")
    if s < 0 or e < 0:
        sys.exit(f"Could not parse JSON from model:\n{text}")
    return json.loads(text[s:e + 1])


def main():
    dry = "--dry-run" in sys.argv
    no_llm = "--no-llm" in sys.argv
    # --final (or STORE_NOTES_FINAL=1): this is the final release, where [Unreleased] has been
    # stamped, so fall back to the newest dated section. Betas must NOT pass it (see unreleased_bullets).
    allow_fallback = "--final" in sys.argv or os.environ.get("STORE_NOTES_FINAL") == "1"
    sources = gather(allow_fallback)
    print("changelog bullets → " +
          ", ".join(f"{s}:{len(v)}" for s, v in sources.items()))
    copy = sources if no_llm else call_anthropic(sources)
    os.makedirs(OUT, exist_ok=True)
    for store in ("android", "apple"):
        lines = [l.strip() for l in copy.get(store, []) if l.strip()]
        if dry or no_llm:
            print(f"\n----- {store}.en.txt -----\n" + "\n".join(lines))
        if not dry:
            with open(os.path.join(OUT, f"{store}.en.txt"), "w", encoding="utf-8") as f:
                f.write("\n".join(lines))
            print(f"{store}: {len(lines)} lines -> out/{store}.en.txt")


if __name__ == "__main__":
    main()
