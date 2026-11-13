#!/usr/bin/env python3
"""Author ONE pull request's changelog entry from its `## Description` (the base).

Runs per-PR in CI, gated on the PR carrying a release milestone. For a milestoned PR:

  1. If the `## Description` is empty or an untouched template, author a description FROM THE DIFF
     internally, to write a good entry. The PR body is never edited (the bot does not mutate the PR).
  2. Author a Keep-a-Changelog entry from the description, using the diff to fill gaps when it is
     thin, for each component CHANGELOG.md the PR's changed paths route to.

In `--suggest` mode (CI) the entry is posted as a committable GitHub suggestion on the dev's
`- _pending changelog_` stub (or a paste comment); without it, the entry is written into the
changelog files (a dev running this locally). This replaces the hand-written changelog entry a dev
used to add. Everything downstream is unchanged: `add-changelog-release` stamps `[Unreleased]` ->
`[version]` at release, and `prepare-release-notes` (`extract-release-notes`) assembles the
GitHub/TestFlight notes. There is no agent in CI — one stateless HTTPS call per task.

Posting/writing is idempotent, keyed by the PR URL (file-write) or a per-component marker
(suggestion): re-runs update this PR's entry in place, and other PRs' entries are untouched.

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
    --suggest   don't commit to the branch: post the entry as a committable GitHub *suggestion* on
                the dev's `- _pending changelog_` stub line (one click to apply), or a paste comment
                if no stub is present. CI uses this; the workflow needs no `contents: write`.
    --dry-run   print what would change; write no files, post nothing, don't edit the PR
    --no-llm    skip the API; stub the entry from the PR title (plumbing smoke test only)

Without --suggest it writes the entry into the component CHANGELOG files (for a dev running it
locally), exactly as before.
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

HEADINGS = ("Added", "Changed", "Fixed", "Security")

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
- Classify into Added / Changed / Fixed / Security (Security = a fix with a security impact). Usually \
ONE line total for a PR; split only if the PR genuinely makes two distinct user-facing changes to \
this component.
- Return ONLY JSON, no prose or code fences:
  {"added":["..."], "changed":["..."], "fixed":["..."], "security":["..."]}"""

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
                  "--json", "number,title,url,body,files,milestone,baseRefName,headRefOid"])
    return {
        "number": pr["number"],
        "title": pr["title"],
        "url": pr["url"],
        "body": pr.get("body") or "",
        "files": [f["path"] for f in pr.get("files", [])],
        "milestone": (pr.get("milestone") or {}).get("title", ""),
        "base": pr.get("baseRefName", ""),
        "head_sha": pr.get("headRefOid", ""),
    }


def pr_diff(number, cap=6000):
    out = subprocess.check_output(["gh", "pr", "diff", str(number), "-R", REPO])
    text = out.decode("utf-8", "replace")
    if len(text) > cap:
        # flag the cut rather than silently dropping the tail, so a thin entry on a big PR is explainable
        return text[:cap] + f"\n\n[diff truncated at {cap} chars; {len(text) - cap} more omitted]"
    return text


# ----------------------------------------------------------------------------- milestone gate

def ms_version(text):
    """The MAJOR.MINOR version embedded in a milestone title or branch name, e.g.
    'v2026.15 - Yari' / 'release/2026.15-yari' -> '2026.15'; '' if none."""
    m = re.search(r"(\d+)\.(\d+)", text or "")
    return f"{m.group(1)}.{m.group(2)}" if m else ""


def ver_tuple(v):
    return tuple(int(x) for x in v.split(".")) if v else ()


def next_release_version():
    """The release develop is heading toward: the lowest open-milestone version that has no
    release/<version> branch yet (i.e. the next one to be cut)."""
    rows = gh_json(["api", f"repos/{REPO}/milestones?state=open&per_page=100"])
    open_vers = {ms_version(r.get("title", "")) for r in rows} - {""}
    out = subprocess.check_output(["git", "ls-remote", "--heads", "origin", "release/*"],
                                  cwd=ROOT).decode()
    cut_vers = {ms_version(l) for l in out.splitlines()} - {""}
    candidates = sorted(open_vers - cut_vers, key=ver_tuple)
    return candidates[0] if candidates else ""


def target_version(base):
    """The release version that PRs merging into `base` belong to: taken from a release/<ver>
    branch name directly, else (develop and the like) the next release to be cut."""
    m = re.match(r"release/(\d+\.\d+)", base or "")
    return m.group(1) if m else next_release_version()


def release_stamped(changelog_text, version_mm):
    """True if this component changelog already has a STAMPED section for `version_mm`
    (`## [<version_mm>...] - <date>` with a real date, not TBD). That is the exact state
    `add-changelog-release` leaves at release close: `[Unreleased]` has rolled over, so authoring a
    PR still milestoned to this version would orphan its entry into the next release."""
    if not version_mm:
        return False
    pat = re.compile(rf"^##\s*\[{re.escape(version_mm)}(?:\.\d+)*\]\s*-\s*(?!TBD\b)\S", re.M)
    return bool(pat.search(changelog_text))


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
        if f"({pr_url})" in ln:     # drop prior bullets for THIS PR — match the parenthesised link
            continue                # the writer appends, so /pull/686 won't match inside /pull/6860
        (sections[cur] if cur else preamble).append(ln)

    # add the new bullets as "- text ([#123](url))": GitHub renders a tidy #123 link, and the dedup
    # above still matches because the "(url)" substring is present inside the markdown link.
    added_n = 0
    for heading in HEADINGS:
        items = [i for i in (buckets.get(heading.lower()) or []) if i.strip()]
        if not items:
            continue
        sections.setdefault(heading, [])
        for it in items:
            sections[heading].append(bullet(it, pr_url))
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


def bullet(text, pr_url):
    """One changelog bullet in the house format: '- <text> ([#N](url))'. The parenthesised URL is
    also the dedup / already-applied key, so upsert and the suggestion must build it identically."""
    num = pr_url.rstrip("/").split("/")[-1]
    return f"- {text.rstrip('.')} ([#{num}]({pr_url}))"


# ----------------------------------------------------------------------------- suggest mode
# Instead of committing the entry onto the PR branch, post it as a committable GitHub *suggestion*
# on the dev's `- _pending changelog_` stub line (one click to apply), or — if no stub is present —
# as a single PR comment to paste. CI never writes to the branch, so the workflow drops
# `contents: write`. The dev places the stub under the heading they judge right (Added/Changed/
# Fixed/Security); the suggestion replaces just that line with the authored bullet, so entries never
# produce a duplicate heading as they accumulate.

STUB_TOKEN = "_pending changelog_"          # the dev drops `- _pending changelog_` under a heading
BOT_MARKER = "changelog-bot"                # hidden per-component marker, for idempotent re-runs


def gh_write(path, payload, method="POST"):
    subprocess.run(["gh", "api", "-X", method, path, "--input", "-"],
                   input=json.dumps(payload).encode("utf-8"), check=True,
                   stdout=subprocess.DEVNULL)


def gh_list(path):
    """GET a paginated list endpoint as a Python list (one compact JSON value per element)."""
    out = subprocess.check_output(["gh", "api", "--paginate", path, "--jq", ".[] | @json"]).decode()
    return [json.loads(line) for line in out.splitlines() if line.strip()]


def pr_file_patches(number):
    """{repo-relative filename: unified-diff patch} for each file GitHub returns a patch for."""
    files = gh_list(f"repos/{REPO}/pulls/{number}/files")
    return {f["filename"]: f.get("patch", "") for f in files}


def stub_line(patch):
    """New-file line number of the dev's `_pending changelog_` stub within this file's diff,
    preferring an added (+) line; None if the stub isn't in the diff (nothing to anchor to)."""
    new_ln, added_hit, ctx_hit = 0, None, None
    for ln in (patch or "").splitlines():
        h = re.match(r"^@@ -\d+(?:,\d+)? \+(\d+)", ln)
        if h:
            new_ln = int(h.group(1))
            continue
        if ln.startswith("+"):
            if STUB_TOKEN in ln and added_hit is None:
                added_hit = new_ln
            new_ln += 1
        elif ln.startswith("-"):
            continue                       # removed line: no new-file number
        elif ln.startswith("\\"):
            continue                       # "\ No newline at end of file": not a file line (D1)
        else:                              # context line
            if STUB_TOKEN in ln and ctx_hit is None:
                ctx_hit = new_ln
            new_ln += 1
    return added_hit if added_hit is not None else ctx_hit


def entry_lines(buckets, pr_url):
    """(flat, grouped): flat tidy bullets for a one-click suggestion (the stub already sits under a
    heading), and {heading: [bullets]} for the paste-fallback comment. Empty if not user-facing."""
    flat, grouped = [], {}
    for heading in HEADINGS:
        for it in [i for i in (buckets.get(heading.lower()) or []) if i.strip()]:
            b = bullet(it, pr_url)
            flat.append(b)
            grouped.setdefault(heading, []).append(b)
    return flat, grouped


def _bot_review_comments(number, marker):
    return [c for c in gh_list(f"repos/{REPO}/pulls/{number}/comments")
            if f"<!-- {marker} -->" in (c.get("body") or "")]


def _bot_issue_comment(number, marker):
    for c in gh_list(f"repos/{REPO}/issues/{number}/comments"):
        if f"<!-- {marker} -->" in (c.get("body") or ""):
            return c
    return None


def _set_issue_comment(number, marker, body, dry):
    """Upsert (body set) or remove (body=None) the single issue comment carrying `marker`."""
    existing = _bot_issue_comment(number, marker)
    if body is None:
        if existing:
            if dry:
                print(f"[dry-run] would delete issue comment {existing['id']} ({marker})")
            else:
                gh_write(f"repos/{REPO}/issues/comments/{existing['id']}", {}, method="DELETE")
        return
    full = f"<!-- {marker} -->\n{body}"
    if dry:
        print(f"[dry-run] would {'update' if existing else 'post'} issue comment ({marker})")
        return
    if existing:
        gh_write(f"repos/{REPO}/issues/comments/{existing['id']}", {"body": full}, method="PATCH")
    else:
        gh_write(f"repos/{REPO}/issues/{number}/comments", {"body": full})


def _drop_review_comments(number, marker, dry):
    for c in _bot_review_comments(number, marker):
        if dry:
            print(f"[dry-run] would delete review comment {c['id']} ({marker})")
        else:
            gh_write(f"repos/{REPO}/pulls/comments/{c['id']}", {}, method="DELETE")


def _suggestion_body(marker, text, block_lines):
    return f"<!-- {marker} -->\n{text}\n\n```suggestion\n" + "\n".join(block_lines) + "\n```"


def post_suggestion(number, head_sha, component, path, line, flat, dry):
    """Replace the stub line with the authored bullet(s) as a one-click suggestion, idempotently
    (a prior suggestion for this component on this PR is removed first)."""
    marker = f"{BOT_MARKER}:{component}"
    name = COMPONENTS[component]["changelog"].split("/")[0]
    text = (f"📝 Proposed **{name}** changelog entry, written from the PR description — click "
            "**Commit suggestion** to add it under this heading. (Edit the PR description to refresh it.)")
    body = _suggestion_body(marker, text, flat)
    if dry:
        print(f"[dry-run] {component}: would suggest on {path}:{line}\n    " + "\n    ".join(flat))
        return
    _drop_review_comments(number, marker, dry)
    gh_write(f"repos/{REPO}/pulls/{number}/comments",
             {"body": body, "commit_id": head_sha, "path": path, "line": line, "side": "RIGHT"})
    print(f"  {component}: suggested entry on {path}:{line}")


def post_stub_removal(number, head_sha, component, path, line, dry):
    """No user-facing entry, but the dev left a stub in the diff — suggest removing the
    `- _pending changelog_` line so it doesn't merge and ship verbatim (D3). An empty suggestion
    block deletes the anchored line."""
    marker = f"{BOT_MARKER}:{component}"
    name = COMPONENTS[component]["changelog"].split("/")[0]
    text = (f"📝 No user-facing **{name}** changelog entry for this PR — click **Commit suggestion** "
            f"to remove the `- {STUB_TOKEN}` placeholder so it doesn't ship.")
    body = _suggestion_body(marker, text, [])
    if dry:
        print(f"[dry-run] {component}: would suggest removing the stub at {path}:{line}")
        return
    _drop_review_comments(number, marker, dry)
    gh_write(f"repos/{REPO}/pulls/{number}/comments",
             {"body": body, "commit_id": head_sha, "path": path, "line": line, "side": "RIGHT"})
    print(f"  {component}: suggested removing the orphaned stub at {path}:{line}")


def upsert_fallback(number, blocks, dry):
    """One PR comment with the entries for components that didn't get a one-click suggestion (no stub,
    or a multi-heading entry that can't go under one heading). Rebuilt each run; removed when none."""
    text = ("📝 **Proposed changelog entries** from the PR description. Add each block under the "
            "matching heading in its `## [Unreleased]` — or, for a one-click suggestion, add a single "
            f"`- {STUB_TOKEN}` stub under that heading. (Edit the PR description to refresh these.)")
    parts = [text]
    for path, grouped in blocks:
        inner = "\n\n".join(f"### {h}\n" + "\n".join(grouped[h]) for h in HEADINGS if grouped.get(h))
        parts.append(f"\n**`{path}`**\n```markdown\n{inner}\n```")
    _set_issue_comment(number, f"{BOT_MARKER}:fallback", "\n".join(parts) if blocks else None, dry)
    if blocks and not dry:
        print(f"  fallback comment covers {len(blocks)} file(s)")


def set_stranded_notice(number, files, dry):
    """PR no longer qualifies but an entry was already applied to the branch — CI can't remove it
    (no write token), so flag it rather than let it ship under the wrong release (D4)."""
    body = None
    if files:
        lst = "\n".join(f"- `{f}`" for f in files)
        body = ("⚠️ This PR no longer carries the release milestone, but a changelog entry for it was "
                "already applied to:\n\n" + lst + "\n\nCI can't remove it (no write access to the "
                "branch). Please delete that entry, or re-milestone the PR, so it doesn't ship under "
                "the wrong release.")
    _set_issue_comment(number, f"{BOT_MARKER}:stranded", body, dry)


def retract_suggestions(number, dry):
    """Remove every suggestion / fallback comment we posted for this PR (used when it stops
    qualifying). Returns how many review comments were removed."""
    n = 0
    for comp in COMPONENTS:
        n += len(_bot_review_comments(number, f"{BOT_MARKER}:{comp}"))
        _drop_review_comments(number, f"{BOT_MARKER}:{comp}", dry)
    _set_issue_comment(number, f"{BOT_MARKER}:fallback", None, dry)
    return n


def already_applied(path_rel, pr_url):
    """True if this PR's entry is already committed on the branch (dev applied the suggestion)."""
    full = os.path.join(ROOT, path_rel)
    return os.path.isfile(full) and f"({pr_url})" in open(full, encoding="utf-8").read()


# ----------------------------------------------------------------------------- main

def main():
    suggest = "--suggest" in sys.argv
    dry = "--dry-run" in sys.argv
    no_llm = "--no-llm" in sys.argv
    number = os.environ.get("PR_NUMBER", "").strip()
    if not number:
        sys.exit("PR_NUMBER is required")

    pr = get_pr(number)

    # --- milestone gate: the PR must be milestoned to THIS release (the one its base branch
    # feeds). Un-milestoned PRs and PRs milestoned to a later release don't qualify. ---
    ms = pr["milestone"]
    reason = ""     # the PR is disqualified -> retract what we posted
    skip = ""       # we can't tell the active release -> leave everything as-is (D5)
    if not ms:
        reason = "has no milestone"
    elif ACTIVE_MILESTONE:                      # explicit override (exact title) wins
        if ms != ACTIVE_MILESTONE:
            reason = f"milestone {ms!r} != active {ACTIVE_MILESTONE!r}"
    else:
        target = target_version(pr["base"])
        if not target:
            skip = f"could not resolve the active release for base {pr['base']!r}"
        elif ms_version(ms) != target:
            reason = f"milestone {ms!r} is not release {target} (base {pr['base']})"

    if skip:
        # Transient: between cutting release N and opening milestone N+1 there is no "next" release to
        # resolve. That is not a disqualification — leave any existing suggestion/entry untouched (D5).
        print(f"::notice::PR #{number} {skip} — skipping (leaving any existing suggestion/entry as-is)")
        return

    if reason:
        # The PR doesn't belong to this release (unmilestoned, milestoned elsewhere, or just
        # demilestoned). Don't author — retract whatever we posted/wrote for it, so a demilestone /
        # re-milestone cleans up instead of leaving a stale suggestion or bullet behind.
        print(f"::notice::PR #{number} {reason} — not authoring; retracting any prior entry")
        if suggest:
            n = retract_suggestions(number, dry)
            # D4: CI can't un-commit an entry the dev already applied (no write token) — flag it so it
            # isn't shipped under a release the PR no longer belongs to.
            stranded = [COMPONENTS[c]["changelog"] for c in sorted(route(pr["files"]))
                        if already_applied(COMPONENTS[c]["changelog"], pr["url"])]
            set_stranded_notice(number, stranded, dry)
            print(f"done: retracted {n} suggestion/comment(s)"
                  + (f"; flagged {len(stranded)} already-applied entry(ies)" if stranded else ""))
            return
        removed = []
        for component in sorted(route(pr["files"])):
            path = os.path.join(ROOT, COMPONENTS[component]["changelog"])
            if not os.path.isfile(path):
                continue
            content = open(path, encoding="utf-8").read()
            # Only touch a changelog that actually has an entry for THIS PR — otherwise skip, so the
            # cleanup never rewrites (or whitespace-churns) a file that had nothing of ours in it.
            if "## [Unreleased]" not in content or f"({pr['url']})" not in content:
                continue
            if upsert(path, {"added": [], "changed": [], "fixed": []}, pr["url"], dry):
                removed.append(COMPONENTS[component]["changelog"])
                print(f"  {component}: removed stale entry")
        print(f"done: removed entry from {len(removed)} changelog file(s)")
        return
    print(f"PR #{number} milestone={ms!r} base={pr['base']} — authoring")

    comps = route(pr["files"])
    print(f"routes to: {', '.join(sorted(comps))}")

    # --- release-close gate: refuse to author into a release that is already stamped (the final
    # `add-changelog-release` has rolled `[Unreleased]` over). Authoring then would orphan this entry
    # into the next release. The stamped, dated section for this version is that signal. ---
    v = ms_version(ms)
    for component in sorted(comps):
        cpath = os.path.join(ROOT, COMPONENTS[component]["changelog"])
        try:
            ctext = open(cpath, encoding="utf-8").read()
        except OSError:
            continue
        if release_stamped(ctext, v):
            print(f"::error::release {v} is already stamped in "
                  f"{COMPONENTS[component]['changelog']} — PR #{number} is milestoned to a closed "
                  f"release. Re-milestone to the next release (or cherry-pick before the stamp); "
                  f"not authoring, to avoid orphaning the entry.")
            sys.exit(1)

    diff = pr_diff(number)

    # --- 1. author the entry's base: if the description is empty/template, derive one from the diff
    # internally. We never edit the PR body (neither mode) — the bot does not mutate the PR (D11). ---
    desc = section(pr["body"], "Description")
    from_diff = is_blank_text(desc)
    if from_diff:
        print(f"::warning::PR #{number} description is empty/template — authoring the entry from the diff")
        desc = pr["title"] if no_llm else author_description(diff)

    # --- 2. author per component, then APPLY: suggest on the stub (CI), or write the file (local) ---
    if suggest:
        patches = pr_file_patches(number)
        blocks = []
        for component in sorted(comps):
            buckets = stub(pr["title"]) if no_llm else \
                author_entry(component, pr["title"], desc, diff)
            flat, grouped = entry_lines(buckets, pr["url"])
            path_rel = COMPONENTS[component]["changelog"]
            marker = f"{BOT_MARKER}:{component}"
            line = stub_line(patches.get(path_rel, ""))

            # already committed on the branch (dev applied it): nothing to do; clear any stale
            # suggestion we left, and don't re-suggest over it (D6).
            if flat and already_applied(path_rel, pr["url"]):
                _drop_review_comments(number, marker, dry)
                print(f"  {component}: entry already applied on the branch")
                continue

            # no user-facing entry: if a stub is anchored in the diff, suggest DELETING it so it can't
            # merge and ship verbatim (D3); otherwise just clear any stale suggestion.
            if not flat:
                if line is not None:
                    post_stub_removal(number, pr["head_sha"], component, path_rel, line, dry)
                else:
                    _drop_review_comments(number, marker, dry)
                    print(f"  {component}: no user-facing entry")
                continue

            # one-click suggestion only when a single heading is populated AND a stub is anchored; a
            # multi-heading entry can't sit as one bullet block under one heading, so it (and the
            # no-stub case) go to the grouped paste comment instead (D2).
            if line is not None and len(grouped) == 1:
                post_suggestion(number, pr["head_sha"], component, path_rel, line, flat, dry)
            else:
                _drop_review_comments(number, marker, dry)
                blocks.append((path_rel, grouped))
                why = "multi-heading entry" if line is not None else "no stub"
                print(f"  {component}: {why} — added to the paste-fallback comment")
        upsert_fallback(number, blocks, dry)
        set_stranded_notice(number, [], dry)   # qualifying again -> clear any prior stranded notice
        if from_diff:
            print(f"::warning::PR #{number} entry authored from a thin description — proofread closely")
        print(f"done: suggested/commented for {len(comps)} component(s)")
        return

    # legacy file-write mode (a dev running this locally): write the entry into the changelog files.
    changed = []
    for component in sorted(comps):
        buckets = stub(pr["title"]) if no_llm else \
            author_entry(component, pr["title"], desc, diff)
        has_entry = any(buckets.get(h.lower()) for h in HEADINGS)
        path = os.path.join(ROOT, COMPONENTS[component]["changelog"])
        # Always upsert: empty buckets REMOVE any prior bullet for this PR (e.g. a PR re-edited to be
        # non-user-facing), keeping [Unreleased] in sync. upsert only reports a change when one moved.
        if upsert(path, buckets, pr["url"], dry):
            changed.append(COMPONENTS[component]["changelog"])
            verb = "upserted" if has_entry else "removed stale entry in"
            print(f"  {component}: {verb} {COMPONENTS[component]['changelog']}")
        elif not has_entry:
            print(f"  {component}: no user-facing entry (nothing to do)")

    if from_diff:
        print(f"::warning::PR #{number} entry authored from diff — proofread closely")
    print(f"done: {len(changed)} changelog file(s) changed")


if __name__ == "__main__":
    main()
