# Release changelog automation

The **PR description is the base.** Instead of a dev hand-writing a `CHANGELOG.md` entry, the
author writes a good `## Description` and the LLM authors the changelog entry from it. Everything
downstream is the plumbing that already exists — this only replaces the hand-written-entry step.

```
PR ## Description ─(author_changelog.py, per PR)→ component CHANGELOG [Unreleased]
                                                        │
                        existing CI, unchanged:         ├─ add-changelog-release  (stamp [Unreleased]→[version] at release)
                                                        └─ prepare-release-notes  (extract-release-notes → GitHub notes + TestFlight)

final release only ──(release-store-notes.yml)→ author_store.py → out/*.en.txt → translate.py → fastlane store files
```

## The two lanes

### 1. Per-PR changelog — `author-pr-changelog.yml` → `author_changelog.py`
Runs on each PR (`opened`, `edited`, `milestoned` — **not** `synchronize`, so code pushes don't
re-author and a reviewer's edits to the entry survive). For **one** PR it:

1. **Milestone gate.** Skips the PR entirely unless it carries a release milestone (the team's
   reliable release-scope signal). `ACTIVE_MILESTONE` can pin an exact title; unset = any milestone.
2. **Repairs the base.** If `## Description` is empty or an untouched template, it authors a
   description **from the diff** and writes it back to the PR body (flagged), so the base is complete
   before anything consumes it.
3. **Authors the entry.** Description-first, backed by the diff (fills gaps when the description is
   thin/vague/jargon, stays conservative), strips internal language, applies the terminology in
   `style.md`, classifies Added/Changed/Fixed, and **upserts** it into each component
   `CHANGELOG.md` `[Unreleased]` the PR's **changed paths** route to.

The entry is committed onto the PR branch — **the PR's own review is the proofread gate; no separate
PR.** Upsert is **idempotent, keyed by the PR URL**: re-runs update this PR's bullet in place and
never touch other PRs' bullets. `[Unreleased]` is therefore always current, so **betas just ship
what's accumulated and cherry-picks carry each entry along — no diff/batch logic anywhere.**

Core `CHANGELOG.md` files are English-only; they are never translated.

### 2. Final-release store notes — `release-store-notes.yml` → `author_store.py` + `translate.py`
**Stage 4 only, never betas.** Derives the app-store "What's new" from the release's changelog,
translates it into every Google Play / App Store locale (DeepL round-1; brand/protocol terms kept
Latin via protect-and-restore; informal register; Android auto-fit ≤500), and opens **one**
`needs-proofread` PR with the fastlane metadata. DeepL is round-1 — **never auto-merge.**

## Release stages

1. **Cut** → branch created; `[Unreleased]` already holds the PR-authored entries.
2. **Beta 1** (public) → ships the accumulated changelog (GitHub notes / TestFlight via the existing CI).
3. **Beta 2, 3, …** → new cherry-picked PRs bring their own entries; nothing extra to run.
4. **Final** → `release-store-notes.yml` produces the translated store copy; human QA at PR approval.

## Secrets

`ANTHROPIC_API_KEY` (a **workspace-scoped** key needs no header; an org-scoped key also needs
`ANTHROPIC_WORKSPACE_ID`), `DEEPL_AUTH_KEY`. Apple store-note *upload* is a separate, deferred piece.

## Run locally

```sh
# per-PR changelog (lane 1)
PR_NUMBER=6876 GH_TOKEN=$(gh auth token) ANTHROPIC_API_KEY=… \
  python scripts/release-changelog/author_changelog.py          # --dry-run / --no-llm available

# final store notes (lane 2)
ANTHROPIC_API_KEY=… python scripts/release-changelog/author_store.py   # writes out/*.en.txt
DEEPL_AUTH_KEY=…    python scripts/release-changelog/translate.py       # writes the fastlane files
```

`--dry-run` (print, write nothing, don't edit the PR) and `--no-llm` (stub, plumbing smoke test) work
on `author_changelog.py` / `author_store.py`.

## Notes

- **Milestone tagging is the gate** — an un-milestoned PR is skipped untouched. A PR milestoned after
  opening is caught by the `milestoned` trigger.
- **Routing is by changed path**; the LLM then drops components where the change isn't user-facing, so
  a cross-cutting PR only lands an entry where it matters.
- **Cutover:** while legacy hand-written `[Unreleased]` entries coexist with LLM ones, an entry whose
  dev wrote the PR link is upgraded in place; one without a link may duplicate — the hand-written step
  is retired, so new releases start clean.
- Targets today's **four** component changelogs; adapts if they're unified (separate work order).
