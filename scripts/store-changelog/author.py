#!/usr/bin/env python3
"""Author English app-store "What's new" copy for a NymVPN release from its merged PRs.

Reads the GitHub milestone's merged PRs (via `gh`), asks the Anthropic Messages API to
produce ranked, user-facing store copy for Android and Apple, and writes one bullet per
line to:
    scripts/store-changelog/out/android.en.txt
    scripts/store-changelog/out/apple.en.txt

NOTE: there is no agent or Claude Code session involved. This is a single, stateless HTTPS
call to the Anthropic Messages API, made by this script on the CI runner.

Env:
    ANTHROPIC_API_KEY  (required)
    MILESTONE          (required, e.g. 'v2026.14 - Xaubub')
    GH_TOKEN           (for `gh`, usually ${{ github.token }})
    ANTHROPIC_MODEL    (optional; pin a dated snapshot in CI for reproducibility)
    REPO               (optional; default nymtech/nym-vpn-client)
"""
import json
import os
import subprocess
import sys
import urllib.request

REPO = os.environ.get("REPO", "nymtech/nym-vpn-client")
MILESTONE = os.environ["MILESTONE"]
MODEL = os.environ.get("ANTHROPIC_MODEL", "claude-opus-4-8")
OUT = os.path.join(os.path.dirname(os.path.abspath(__file__)), "out")

SYSTEM = """You write concise app-store "What's new" release notes for the NymVPN apps \
from a list of merged pull requests.

Rules:
- Include ONLY user-facing changes. Exclude CI, build, tests, refactors, dependency bumps, \
internal plumbing, and anything with no visible effect for users.
- Rank bullets by user impact: new features first, then notable fixes (security, \
connectivity, usability), then minor changes.
- Rewrite in plain, benefit-oriented language. Do NOT paste PR titles or use developer wording.
- Android (Google Play) must stay well under 500 characters even after ~60% expansion in \
translation, so keep it very tight: aim for <= 320 characters, about 5 short bullets.
- Apple (App Store) has more room (4000 chars) but stay concise: about 7 bullets max.
- Core/engine changes affect every platform; surface them on both where user-visible.
- Plain sentences only: no bullet characters, no markdown, no trailing punctuation beyond a period.
- Return ONLY JSON: {"android": ["line", ...], "apple": ["line", ...]}. No prose, no code fences."""


def merged_prs():
    out = subprocess.check_output([
        "gh", "pr", "list", "-R", REPO, "--state", "merged", "--limit", "300",
        "--search", f'milestone:"{MILESTONE}"',
        "--json", "number,title,labels",
    ])
    rows = json.loads(out)
    return [
        {"number": r["number"], "title": r["title"],
         "labels": [l["name"] for l in r.get("labels", [])]}
        for r in rows
    ]


def call_anthropic(prs):
    body = {
        "model": MODEL,
        "max_tokens": 1500,
        "system": SYSTEM,
        "messages": [{
            "role": "user",
            "content": f"Milestone: {MILESTONE}\nMerged PRs:\n"
                       f"{json.dumps(prs, ensure_ascii=False)}",
        }],
    }
    req = urllib.request.Request(
        "https://api.anthropic.com/v1/messages",
        data=json.dumps(body).encode("utf-8"),
        headers={
            "x-api-key": os.environ["ANTHROPIC_API_KEY"],
            "anthropic-version": "2023-06-01",
            "content-type": "application/json",
        },
    )
    with urllib.request.urlopen(req, timeout=120) as resp:
        data = json.load(resp)
    text = "".join(block.get("text", "") for block in data["content"])
    start, end = text.find("{"), text.rfind("}")
    if start < 0 or end < 0:
        sys.exit(f"Could not parse JSON from model response:\n{text}")
    return json.loads(text[start:end + 1])


def main():
    prs = merged_prs()
    if not prs:
        sys.exit(f"No merged PRs found for milestone {MILESTONE!r}")
    copy = call_anthropic(prs)
    os.makedirs(OUT, exist_ok=True)
    for platform in ("android", "apple"):
        lines = [l.strip() for l in copy.get(platform, []) if l.strip()]
        path = os.path.join(OUT, f"{platform}.en.txt")
        with open(path, "w", encoding="utf-8") as f:
            f.write("\n".join(lines))
        print(f"{platform}: {len(lines)} bullets -> {path}")


if __name__ == "__main__":
    main()
