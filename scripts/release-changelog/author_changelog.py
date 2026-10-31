#!/usr/bin/env python3
"""Author ONE pull request's changelog entry from its `## Description` (the base).

Runs per-PR in CI, gated on the PR carrying a release milestone. For a milestoned PR:

  1. If the `## Description` is empty or an untouched template, author a description FROM THE
     DIFF and write it back to the PR body (flagged) — repair the base before anything consumes it.
  2. Author a Keep-a-Changelog entry from the (now-complete) description, using the diff to fill
     gaps when the description is thin, and upsert it into the `## [Unreleased]` section of each
     component CHANGELOG.md the PR's changed paths route to.

This replaces the hand-written changelog entry a dev used to add (the `- [ ] Changelog` checkbox).
Everything downstream is unchanged: `add-changelog-release` stamps `[Unreleased]` -> `[version]`
at release, and `prepare-release-notes` (`extract-release-notes`) assembles the GitHub/TestFlight
notes. There is no agent in CI — one stateless HTTPS call per task.

Upsert is idempotent: entries are keyed by the PR URL, so re-runs (on `synchronize`) update this
PR's bullet in place instead of duplicating it, and other PRs' bullets are left untouched.

Env:
    PR_NUMBER          (required) the PR to author for
    ANTHROPIC_API_KEY  (required unless --no-llm)
    GH_TOKEN           (for `gh`)
    ANTHROPIC_MODEL    (optional; pin a dated snapshot in CI)
    ANTHROPIC_WORKSPACE_ID (optional; only for org-scoped keys)
    REPO               (optional; default nymtech/nym-vpn-client)
    ACTIVE_MILESTONE   (optional) require the PR's milestone to equal this exact title; if unset,
                       any non-empty milestone qualifies (the team tags release milestones reliably)

Flags:
    --dry-run   print what would change; write no files and don't edit the PR
    --no-llm    skip the API; stub the entry from the PR title (plumbing smoke test only)
"""
import json
import os
import re
import subprocess
import sys
import urllib.request

REPO = os.environ.get("REPO", "nymtech/nym-vpn-client")
MODEL = os.environ.get("ANTHROPIC_MODEL", "claude-opus-4-8")
ROOT = os.environ.get("GITHUB_WORKSPACE", os.getcwd())
HERE = os.path.dirname(os.path.abspath(__file__))
ACTIVE_MILESTONE = os.environ.get("ACTIVE_MILESTONE", "").strip()

HEADINGS = ("Added", "Changed", "Fixed")

# component -> its CHANGELOG file and the source dirs whose changes route a PR to it.
# Routing is by CHANGED PATHS (reliable); labels are not used (most PRs carry none).
COMPONENTS = {
    "core":    {"changelog": "nym-vpn-core/CHANGELOG.md",    "dirs": ["nym-vpn-core/"]},
    "android": {"changelog": "nym-vpn-android/CHANGELOG.md",  "dirs": ["nym-vpn-android/"]},
    "app":     {"changelog": "nym-vpn-app/CHANGELOG.md",      "dirs": ["nym-vpn-app/"]},
    "apple":   {"changelog": "nym-vpn-apple/CHANGELOG.md",    "dirs": ["nym-vpn-apple/"]},
}

# Placeholder phrases from the PR template; a line containing one is boilerplate, not a real
# description, and is dropped when deciding whether `## Description` is blank.
TEMPLATE_PLACEHOLDERS = ("JIRA-NYM-XXXX", "Replace `XXXX`", "Please include a summary",
                         "Please also include relevant motivation", "Linking this PR does not count",
                         "optional, if UI related")


def load_style():
    try:
        return open(os.path.join(HERE, "style.md"), encoding="utf-8").read().strip()
    except OSError:
        return ""


STYLE = load_style()

ENTRY_SYSTEM = """You write ONE developer changelog entry for the __COMPONENT__ component of the \
NymVPN client, from a single merged pull request. You are an EDITOR, not a pipe: the developer's \
description is your INPUT, never your output — a dev's unthinking internal wording is a prompt, \
not the answer.

Rules:
- Include the change only if it is user-facing or otherwise notable. If it is pure CI, build, \
tests, a dependency bump, or internal refactor/plumbing with no notable effect, return empty lists.
- STRIP internal language. No code symbols or type/function/module names \
(e.g. `Socks5ServiceState::cleanup()`), no ticket ids, no dev shorthand. Rewrite the change as one \
clear sentence a technical user understands.
- Use the project's established product terms (see the terminology below), matching how the \
existing changelog already names the same feature/setting. Never invent ad-hoc phrasing.
- The description is the base. If it is thin, vague, or internal-only, use the diff_excerpt to \
write an accurate entry — but be conservative: never claim more than the diff proves.
- This is the technical/developer changelog (feeds GitHub release notes + TestFlight), NOT \
app-store copy. Clear, precise, technical-but-readable — not marketing.
- Classify into Added / Changed / Fixed. Usually ONE line total for a PR; split only if the PR \
genuinely makes two distinct user-facing changes to this component.
- Return ONLY JSON, no prose or code fences:
  {"added":["..."], "changed":["..."], "fixed":["..."]}"""

DESC_SYSTEM = """You write a pull request description from its diff, because the author left it \
empty or only the template. Write 2–4 plain sentences: what observably changed and, if the diff \
makes it clear, why. Technical but readable. No code symbols, type/function names, file paths, or \
ticket ids. Be conservative — describe only what the diff proves. Return ONLY the description \
text, no heading, no preamble, no code fences."""


# ----------------------------------------------------------------------------- gh helpers

def gh_json(args):
    return json.loads(subprocess.check_output(["gh", *args]))


def get_pr(number):
    pr = gh_json(["pr", "view", str(number), "-R", REPO,
                  "--json", "number,title,url,body,files,milestone"])
    return {
        "number": pr["number"],
        "title": pr["title"],
        "url": pr["url"],
        "body": pr.get("body") or "",
        "files": [f["path"] for f in pr.get("files", [])],
        "milestone": (pr.get("milestone") or {}).get("title", ""),
    }


def pr_diff(number, cap=6000):
    out = subprocess.check_output(["gh", "pr", "diff", str(number), "-R", REPO])
    return out.decode("utf-8", "replace")[:cap]


def set_pr_body(number, body, dry):
    if dry:
        print(f"[dry-run] would update PR #{number} body ({len(body)} chars)")
        return
    subprocess.run(["gh", "pr", "edit", str(number), "-R", REPO, "--body", body], check=True)
    print(f"::notice::backfilled PR #{number} description from diff")


# ----------------------------------------------------------------------------- description

def section(body, name):
    """Text of a `## <name>` section up to the next `## ` heading, or '' if absent."""
    m = re.search(rf"^##+\s*{re.escape(name)}\s*\n(.*?)(?=^##\s|\Z)", body, re.S | re.M | re.I)
    return m.group(1).strip() if m else ""


def is_blank_text(text):
    """True if `text` is empty or only template boilerplate / checkboxes."""
    text = re.sub(r"<!--.*?-->", "", text, flags=re.S)
    kept = []
    for ln in text.splitlines():
        s = ln.strip()
        if not s or s == "---" or s.startswith("#"):
            continue
        if s.startswith("- [") or re.fullmatch(r"[-*]", s):
            continue
        if any(ph in s for ph in TEMPLATE_PLACEHOLDERS):
            continue
        kept.append(s)
    return len(" ".join(kept)) < 20


def replace_section(body, name, new_text):
    """Replace the body of a `## <name>` section, or prepend one if it doesn't exist."""
    pat = re.compile(rf"(^##+\s*{re.escape(name)}\s*\n)(.*?)(?=^##\s|\Z)", re.S | re.M | re.I)
    if pat.search(body):
        return pat.sub(lambda m: f"{m.group(1)}\n{new_text}\n\n", body, count=1)
    head = f"## {name}\n\n{new_text}\n"
    return f"{head}\n{body}" if body.strip() else head


# ----------------------------------------------------------------------------- anthropic

def call_anthropic(system, user, max_tokens=1500):
    body = {"model": MODEL, "max_tokens": max_tokens, "system": system,
            "messages": [{"role": "user", "content": user}]}
    headers = {"x-api-key": os.environ["ANTHROPIC_API_KEY"],
               "anthropic-version": "2023-06-01", "content-type": "application/json"}
    ws = os.environ.get("ANTHROPIC_WORKSPACE_ID")
    if ws:
        headers["anthropic-workspace-id"] = ws
    req = urllib.request.Request("https://api.anthropic.com/v1/messages",
                                 data=json.dumps(body).encode("utf-8"), headers=headers)
    with urllib.request.urlopen(req, timeout=120) as resp:
        data = json.load(resp)
    return "".join(b.get("text", "") for b in data["content"]).strip()


def author_description(diff):
    return call_anthropic(DESC_SYSTEM, f"Diff:\n{diff}", max_tokens=600)


def author_entry(component, title, description, diff):
    system = ENTRY_SYSTEM.replace("__COMPONENT__", component) + (f"\n\n{STYLE}" if STYLE else "")
    user = (f"Component: {component}\nPR title: {title}\n\n"
            f"Description (the base):\n{description}\n\nDiff excerpt:\n{diff}")
    text = call_anthropic(system, user, max_tokens=1200)
    s, e = text.find("{"), text.rfind("}")
    if s < 0 or e < 0:
        sys.exit(f"[{component}] could not parse JSON from model:\n{text}")
    return json.loads(text[s:e + 1])


# ----------------------------------------------------------------------------- routing + upsert

def route(files):
    hits = {c for c, cfg in COMPONENTS.items()
            if any(p.startswith(d) for d in cfg["dirs"] for p in files)}
    return hits or {"core"}


def upsert(path, buckets, pr_url, dry):
    """Idempotently place this PR's bullets in `## [Unreleased]`, keyed by pr_url.

    Removes any existing lines carrying this pr_url (so re-runs update in place), then appends the
    new bullets under their heading. Other PRs' bullets are untouched; headings keep Added/Changed/
    Fixed order. Returns True if the file content changed.
    """
    with open(path, encoding="utf-8") as f:
        text = f.read()
    m = re.search(r"(## \[Unreleased\]\n)(.*?)(\n## \[|\Z)", text, re.S)
    if not m:
        sys.exit(f"{path}: no '## [Unreleased]' section")
    head, body, tail = m.group(1), m.group(2), m.group(3)

    # parse existing [Unreleased] into preamble + {heading: [bullet lines]}, dropping this PR's lines
    preamble, sections, cur = [], {}, None
    for ln in body.splitlines():
        hm = re.match(r"^###\s+(.+?)\s*$", ln)
        if hm:
            cur = hm.group(1)
            sections.setdefault(cur, [])
            continue
        if pr_url in ln:            # drop prior bullets for this PR (dedup / in-place update)
            continue
        (sections[cur] if cur else preamble).append(ln)

    # add the new bullets
    added_n = 0
    for heading in HEADINGS:
        items = [i for i in (buckets.get(heading.lower()) or []) if i.strip()]
        if not items:
            continue
        sections.setdefault(heading, [])
        for it in items:
            sections[heading].append(f"- {it.rstrip('.')} ({pr_url})")
            added_n += 1

    # re-render: canonical heading order first, then any others, preserving bullet order
    order = [h for h in HEADINGS if sections.get(h)] + \
            [h for h in sections if h not in HEADINGS and sections.get(h)]
    parts = []
    pre = "\n".join(preamble).strip("\n")
    if pre:
        parts.append(pre)
    for h in order:
        lines = [l for l in sections[h] if l.strip()]
        if lines:
            parts.append(f"### {h}\n" + "\n".join(lines))
    new_body = "\n\n" + "\n\n".join(parts) + "\n" if parts else "\n"
    new_text = text[:m.start()] + head + new_body + tail + text[m.end():]

    if new_text == text:
        return False
    if dry:
        print(f"[dry-run] {path}: would set this PR's {added_n} bullet(s) in [Unreleased]")
        return True
    with open(path, "w", encoding="utf-8") as f:
        f.write(new_text)
    return True


def stub(title):
    return {"added": [], "changed": [title], "fixed": []}


# ----------------------------------------------------------------------------- main

def main():
    dry = "--dry-run" in sys.argv
    no_llm = "--no-llm" in sys.argv
    number = os.environ.get("PR_NUMBER", "").strip()
    if not number:
        sys.exit("PR_NUMBER is required")

    pr = get_pr(number)

    # --- milestone gate ---
    ms = pr["milestone"]
    if not ms:
        print(f"::notice::PR #{number} has no milestone — skipping changelog authoring")
        return
    if ACTIVE_MILESTONE and ms != ACTIVE_MILESTONE:
        print(f"::notice::PR #{number} milestone {ms!r} != active {ACTIVE_MILESTONE!r} — skipping")
        return
    print(f"PR #{number} milestone={ms!r} — authoring")

    diff = pr_diff(number)

    # --- 1. repair the base: backfill an empty/template description ---
    desc = section(pr["body"], "Description")
    from_diff = is_blank_text(desc)
    if from_diff:
        print(f"::warning::PR #{number} description is empty/template — authoring from diff")
        if no_llm:
            desc = pr["title"]
        else:
            desc = author_description(diff)
            new_body = replace_section(pr["body"], "Description", desc)
            set_pr_body(number, new_body, dry)

    # --- 2. author the changelog entry per routed component ---
    comps = route(pr["files"])
    print(f"routes to: {', '.join(sorted(comps))}")
    changed = []
    for component in sorted(comps):
        buckets = stub(pr["title"]) if no_llm else \
            author_entry(component, pr["title"], desc, diff)
        if not any(buckets.get(h.lower()) for h in HEADINGS):
            print(f"  {component}: no user-facing entry (skipped)")
            continue
        path = os.path.join(ROOT, COMPONENTS[component]["changelog"])
        if upsert(path, buckets, pr["url"], dry):
            changed.append(COMPONENTS[component]["changelog"])
            print(f"  {component}: upserted -> {COMPONENTS[component]['changelog']}")

    if from_diff:
        print(f"::warning::PR #{number} entry authored from diff — proofread closely")
    # emit changed files for the workflow to commit
    gh_out = os.environ.get("GITHUB_OUTPUT")
    if gh_out and not dry:
        with open(gh_out, "a", encoding="utf-8") as f:
            f.write(f"changed={' '.join(changed)}\n")
            f.write(f"backfilled={'true' if from_diff else 'false'}\n")
    print(f"done: {len(changed)} changelog file(s) changed")


if __name__ == "__main__":
    main()
