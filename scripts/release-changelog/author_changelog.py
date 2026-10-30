#!/usr/bin/env python3
"""(a) Author the component CHANGELOG `[Unreleased]` sections from a milestone's merged PRs.

Flow: gather the milestone's merged PRs (via `gh`) -> route each to one or more
components by its `affects:*` labels -> ask the Anthropic Messages API to write
Keep-a-Changelog bullets per component -> inject them into each component's
`## [Unreleased]` section. The existing release CI (`add-changelog-release` stamp +
`ffurrer2/extract-release-notes`) consumes those sections unchanged.

There is no agent/Claude Code session in CI: one stateless HTTPS call per component.

Env:
    MILESTONE          (required, e.g. 'v2026.15 - Yari')
    ANTHROPIC_API_KEY  (required unless --no-llm)
    GH_TOKEN           (for `gh`)
    ANTHROPIC_MODEL    (optional; pin a dated snapshot in CI)
    REPO               (optional; default nymtech/nym-vpn-client)

Flags:
    --dry-run   print the rendered sections, write nothing
    --no-llm    skip the API; render raw PR titles as bullets (smoke test of
                gather+route+inject only — NOT real changelog copy)
"""
import json
import os
import re
import subprocess
import sys
import urllib.request

REPO = os.environ.get("REPO", "nymtech/nym-vpn-client")
MILESTONE = os.environ.get("MILESTONE", "")
MODEL = os.environ.get("ANTHROPIC_MODEL", "claude-opus-4-8")
ROOT = os.environ.get("GITHUB_WORKSPACE", os.getcwd())
HERE = os.path.dirname(os.path.abspath(__file__))


def load_style():
    """The committed house style/terminology, loaded into the author prompt."""
    try:
        return open(os.path.join(HERE, "style.md"), encoding="utf-8").read().strip()
    except OSError:
        return ""


STYLE = load_style()

# component -> its CHANGELOG file, the source dirs whose changes route a PR to it,
# and the affects:* labels used only as a fallback when a PR touches no component dir.
# Routing is primarily by CHANGED PATHS: labels are applied too inconsistently to rely on
# (most PRs carry no affects:* label), so path is the signal that reflects what actually moved.
COMPONENTS = {
    "core":    {"changelog": "nym-vpn-core/CHANGELOG.md",   "dirs": ["nym-vpn-core/"],    "labels": []},
    "android": {"changelog": "nym-vpn-android/CHANGELOG.md", "dirs": ["nym-vpn-android/"], "labels": ["affects:android"]},
    "app":     {"changelog": "nym-vpn-app/CHANGELOG.md",     "dirs": ["nym-vpn-app/"],     "labels": ["affects:windows", "affects:linux"]},
    "apple":   {"changelog": "nym-vpn-apple/CHANGELOG.md",   "dirs": ["nym-vpn-apple/"],   "labels": ["affects:ios", "affects:macos"]},
}

SYSTEM = """You write the developer release changelog for the {component} component of the \
NymVPN client, from a list of merged pull requests. You are an EDITOR, not a pipe: a developer's \
raw description is your INPUT, never your output — a dev's unthinking internal wording is a prompt, \
not the answer.

Rules:
- Include only user-facing or otherwise notable changes. Exclude pure CI, build, tests, \
dependency bumps, refactors and internal plumbing unless genuinely notable.
- STRIP internal language. No code symbols or type/function/module names \
(e.g. `Socks5ServiceState::cleanup()`), no ticket ids, no dev shorthand. Rewrite the underlying \
change as one clear sentence a technical user understands.
- Be CONSISTENT across the whole set: name the same feature/setting/component the same way in every \
entry, matching how the existing changelog already names it. Use the project's established product \
terms, never ad-hoc per-dev phrasing. Fix terminology drift between entries.
- If a PR has an empty `release_note`, author the entry from its `diff_excerpt` instead — describe \
the observable change, and be conservative (don't overstate what the diff proves).
- This is the technical/developer changelog (feeds GitHub release notes + TestFlight), NOT \
app-store copy. Clear, precise, technical-but-readable — not marketing.
- Classify each change into exactly one of Added / Changed / Fixed. One line per change. \
Don't invent detail beyond the PR's material.
- Preserve the PR link.
- Return ONLY JSON, no prose or code fences:
  {"added":[{"text":"...","pr":"<url>"}], "changed":[...], "fixed":[...]}"""

# Placeholder phrases from the PR template; a line containing one is template boilerplate,
# not a real description, and is dropped when deciding whether a body is blank.
TEMPLATE_PLACEHOLDERS = ("JIRA-NYM-XXXX", "Replace `XXXX`", "Please include a summary",
                         "Please also include relevant motivation", "Linking this PR does not count",
                         "optional, if UI related")


def gather_prs():
    out = subprocess.check_output([
        "gh", "pr", "list", "-R", REPO, "--state", "merged", "--limit", "300",
        "--search", f'milestone:"{MILESTONE}"',
        "--json", "number,title,labels,body,url",
    ])
    rows = json.loads(out)
    prs = []
    for r in rows:
        body = r.get("body") or ""
        note = extract_release_note(body)
        # no release note AND a blank/template description -> author from the diff, and flag it
        from_diff = not note and is_blank_body(body)
        prs.append({
            "number": r["number"],
            "title": r["title"],
            "url": r["url"],
            "labels": [l["name"] for l in r.get("labels", [])],
            "files": pr_files(r["number"]),
            "note": note,
            "from_diff": from_diff,
            "diff": pr_diff(r["number"]) if from_diff else "",
        })
    return prs


def is_blank_body(body):
    """True if the PR body is empty or only the untouched template (no real description)."""
    text = re.sub(r"<!--.*?-->", "", body, flags=re.S)                 # HTML comments
    text = re.sub(r"(?is)(##\s*Summary by CodeRabbit|Reviewable:).*", "", text)  # bot blocks
    kept = []
    for ln in text.splitlines():
        s = ln.strip()
        if not s or s == "---":
            continue
        if s.startswith("#"):                                          # headings
            continue
        if s.startswith("- [") or re.fullmatch(r"[-*]", s):            # checkboxes / empty bullets
            continue
        if any(ph in s for ph in TEMPLATE_PLACEHOLDERS):               # template placeholder lines
            continue
        kept.append(s)
    return len(" ".join(kept)) < 20


def pr_diff(number, cap=6000):
    """Capped unified diff, used to author an entry when the description is blank."""
    out = subprocess.check_output(["gh", "pr", "diff", str(number), "-R", REPO])
    return out.decode("utf-8", "replace")[:cap]


def pr_files(number):
    """Changed file paths for a PR (the reliable routing signal)."""
    out = subprocess.check_output([
        "gh", "pr", "view", str(number), "-R", REPO, "--json", "files"])
    return [f["path"] for f in json.loads(out).get("files", [])]


def extract_release_note(body):
    """Pull a `## Release note` section or `Release note:` line from the PR body, else ''."""
    m = re.search(r"##+\s*Release note\s*\n+(.+?)(?:\n#{1,6}\s|\Z)", body, re.S | re.I)
    if m:
        return m.group(1).strip()
    m = re.search(r"Release note:\s*(.+)", body, re.I)
    return m.group(1).strip() if m else ""


def route(pr):
    """Components this PR belongs to, by changed path; fall back to affects:* labels, else core."""
    hits = {c for c, cfg in COMPONENTS.items()
            if any(p.startswith(d) for d in cfg["dirs"] for p in pr["files"])}
    if not hits:
        hits = {c for c, cfg in COMPONENTS.items()
                if cfg["labels"] and any(lbl in pr["labels"] for lbl in cfg["labels"])}
    return hits or {"core"}


def call_anthropic(component, prs):
    payload = [{"number": p["number"], "title": p["title"], "url": p["url"],
                "release_note": p["note"],
                "diff_excerpt": p["diff"] if p["from_diff"] else ""} for p in prs]
    body = {
        "model": MODEL,
        "max_tokens": 2000,
        "system": SYSTEM.format(component=component) + (f"\n\n{STYLE}" if STYLE else ""),
        "messages": [{"role": "user",
                      "content": f"Milestone: {MILESTONE}\nComponent: {component}\n"
                                 f"Merged PRs:\n{json.dumps(payload, ensure_ascii=False)}"}],
    }
    req = urllib.request.Request(
        "https://api.anthropic.com/v1/messages",
        data=json.dumps(body).encode("utf-8"),
        headers={"x-api-key": os.environ["ANTHROPIC_API_KEY"],
                 "anthropic-version": "2023-06-01", "content-type": "application/json"})
    with urllib.request.urlopen(req, timeout=120) as resp:
        data = json.load(resp)
    text = "".join(b.get("text", "") for b in data["content"])
    s, e = text.find("{"), text.rfind("}")
    if s < 0 or e < 0:
        sys.exit(f"[{component}] could not parse JSON from model:\n{text}")
    return json.loads(text[s:e + 1])


def stub(prs):
    """--no-llm: group raw titles under Changed, for a gather/route/inject smoke test only."""
    return {"added": [], "changed": [{"text": p["title"], "pr": p["url"]} for p in prs],
            "fixed": []}


def render_section(buckets):
    parts = []
    for heading in ("Added", "Changed", "Fixed"):
        items = buckets.get(heading.lower()) or []
        if not items:
            continue
        parts.append(f"### {heading}")
        for it in items:
            pr = f" ({it['pr']})" if it.get("pr") else ""
            parts.append(f"- {it['text'].rstrip('.')}{pr}")
        parts.append("")
    return "\n".join(parts).rstrip() + "\n" if parts else ""


def inject_unreleased(path, section_body):
    """Replace the body of `## [Unreleased]` (up to the next `## [`) with section_body."""
    with open(path, encoding="utf-8") as f:
        text = f.read()
    if "## [Unreleased]" not in text:
        sys.exit(f"{path}: no '## [Unreleased]' section")
    pat = re.compile(r"(## \[Unreleased\]\n)(.*?)(\n## \[)", re.S)
    repl = f"\\g<1>\n{section_body}\n\\g<3>" if section_body else r"\g<1>\g<3>"
    new, n = pat.subn(repl, text, count=1)
    if n == 0:  # [Unreleased] is the last section
        new = re.sub(r"(## \[Unreleased\]\n)", f"\\g<1>\n{section_body}", text, count=1)
    with open(path, "w", encoding="utf-8") as f:
        f.write(new)


def main():
    dry = "--dry-run" in sys.argv
    no_llm = "--no-llm" in sys.argv
    if not MILESTONE:
        sys.exit("MILESTONE is required")

    prs = gather_prs()
    if not prs:
        sys.exit(f"No merged PRs for milestone {MILESTONE!r}")

    by_component = {c: [] for c in COMPONENTS}
    for pr in prs:
        for c in route(pr):
            by_component[c].append(pr)

    print(f"{len(prs)} merged PRs → " +
          ", ".join(f"{c}:{len(v)}" for c, v in by_component.items()))

    flagged = [p["number"] for p in prs if p["from_diff"]]
    if flagged:
        print(f"::warning::{len(flagged)} PR(s) had no description — authored from diff, "
              f"proofread closely: {flagged}")

    for component, cfg in COMPONENTS.items():
        cprs = by_component[component]
        if not cprs:
            continue
        buckets = stub(cprs) if no_llm else call_anthropic(component, cprs)
        section = render_section(buckets)
        path = os.path.join(ROOT, cfg["changelog"])
        if dry or no_llm:
            print(f"\n----- {component}: {cfg['changelog']} -----\n{section}", end="")
        if not dry:
            inject_unreleased(path, section)
            print(f"wrote {component} → {cfg['changelog']}")


if __name__ == "__main__":
    main()
