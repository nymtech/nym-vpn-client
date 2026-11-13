# Release changelog automation

The **PR description is the base.** Instead of a dev hand-writing a `CHANGELOG.md` entry, the author
writes a good `## Description`, the LLM drafts the changelog entry from it, and the bot posts it as a
one-click GitHub **suggestion** on the PR — the dev applies it with a click. CI never commits to the
branch. Everything downstream is the plumbing that already exists; this only replaces the
hand-written-entry step.

```
PR ## Description ─(author_changelog.py --suggest)→ one-click suggestion → dev applies → [Unreleased]
                                                        │
                        existing CI, unchanged:         ├─ add-changelog-release  (stamp [Unreleased]→[version] at release)
                                                        └─ prepare-release-notes  (extract-release-notes → GitHub notes + TestFlight)

final release only ──(release-store-notes.yml)→ author_store.py → out/*.en.txt → translate.py → fastlane store files
```

## The two lanes

### 1. Per-PR changelog — `author-pr-changelog.yml` → `author_changelog.py --suggest`
Runs on each PR (`opened`, `edited`, `milestoned`, `demilestoned`, `ready_for_review` — **not**
`synchronize`, so code pushes don't re-draft; `demilestoned` retracts the suggestion a PR no longer
qualifies for). For **one** PR it:

1. **Milestone gate.** Skips the PR entirely unless it carries a release milestone (the team's
   reliable release-scope signal). `ACTIVE_MILESTONE` can pin an exact title; unset = any milestone.
2. **Reads the base.** If `## Description` is empty or an untouched template, it authors a
   description **from the diff** internally to write a good entry — but **does not** edit the PR body
   (CI never mutates the PR).
3. **Drafts the entry.** Description-first, backed by the diff (fills gaps when the description is
   thin/vague/jargon, stays conservative), strips internal language, applies the terminology in
   `style.md`, classifies Added/Changed/Fixed/Security, for each component `CHANGELOG.md` the PR's **changed
   paths** route to.
4. **Posts it as a suggestion.** On the dev's `- _pending changelog_` stub line (placed under the
   heading they judge right), the bot posts a committable GitHub **suggestion** — one click applies it
   as the dev's own commit. No stub? It posts one PR comment with the entry to paste. **CI never
   writes to the branch**, so the workflow needs no `contents: write`.

**The human applying the suggestion is the proofread gate; no separate PR.** Posting is **idempotent,
keyed by a per-component marker**: re-runs replace this PR's prior suggestion, a demilestone retracts
it, and placing the stub under a heading keeps entries from ever producing a duplicate heading as they
accumulate. `[Unreleased]` stays current, so **betas just ship what's accumulated and cherry-picks
carry each entry along — no diff/batch logic anywhere.**

> **Local option.** Run `author_changelog.py` **without** `--suggest` and it writes the entry straight
> into the changelog files instead — for a dev who'd rather generate it locally and commit it.

**Release-close gate.** The authoring step refuses to author into a release that is already stamped
(the final `add-changelog-release` has rolled `[Unreleased]` over into `## [version] - date`).
A PR still milestoned to that closed release fails with a clear message to re-milestone (or
cherry-pick before the stamp), rather than silently orphaning its entry into the next release.

Core `CHANGELOG.md` files are English-only; they are never translated.

> **One stamper.** The app changelog is stamped only by the unified `add-changelog-release`
> (via `bump-all-platforms`), like the other three components. `tauri-post-release.yml` no longer
> touches any `CHANGELOG.md` — it only bumps the Linux installer + flatpak metainfo now.

### 2. Final-release store notes — `release-store-notes.yml` → `author_store.py` + `translate.py`
**Stage 4 only, never betas.** Derives the app-store "What's new" from the release's changelog,
translates it into every Google Play / App Store locale (DeepL round-1; brand/protocol terms kept
Latin via protect-and-restore; informal register; Android auto-fit ≤500), runs the fastlane length
guard (below), and opens **one** `needs-proofread` PR with the fastlane metadata. DeepL is round-1 —
**never auto-merge.**

### 3. Beta TestFlight notes — `release-testflight-notes.yml` → `publish-nym-vpn-ios.yml`
TestFlight is a **beta (public test)** asset, so its "what's new" is **human-reviewed before Beta 1**.
When a release branch is cut, `release-testflight-notes.yml` drafts the Apple consumer note from the
release's already-PR-reviewed `[Unreleased]` (`author_store.py`, English) and opens one
`needs-proofread` PR writing `nym-vpn-apple/fastlane/TestFlight/whats-new.txt` — **never auto-merged**.
A person proofreads + approves it; only then does `publish-nym-vpn-ios.yml` ship that **approved file**
to testers (`--whats-new`). The LLM never runs in the publish path, and with no approved note present
it falls back to the build-metadata stub (never an unreviewed AI note). **Android does not** publish
store copy at betas, so this is Apple-only; full translated store copy for both stores still comes
from lane 2 at final.

### Guard — `check-fastlane-metadata.yaml` → `check_fastlane_limits.py`
One check for the **whole** store-metadata surface, not just changelogs: every length-limited
fastlane field (Play title/short/full/changelog; App Store name/subtitle/promo/keywords/description/
release-notes) against its store limit. Runs on PRs to **develop and `release/**`** (the store-notes
PR targets a release branch), and the same check runs in-generator in `release-store-notes.yml`
before it opens its PR. Replaces the old changelog-only, develop-only `check-changelog.yaml`.

## Release stages

1. **Cut** → branch created; `[Unreleased]` already holds the PR-authored entries.
2. **Beta 1** (public) → ships the accumulated changelog (GitHub notes via the existing CI); the
   TestFlight "what's new" is drafted at cut and **proofread + approved before Beta 1 is pushed** (lane 3).
3. **Beta 2, 3, …** → new cherry-picked PRs bring their own entries; refresh/re-approve the TestFlight
   note if it changed; nothing else to run.
4. **Final** → `release-store-notes.yml` produces the translated store copy; human QA at PR approval.
   After the final stamp, the release-close gate rejects any PR still milestoned to it.

## Secrets

`ANTHROPIC_API_KEY` (a **workspace-scoped** key needs no header; an org-scoped key also needs
`ANTHROPIC_WORKSPACE_ID`) — used by lane 1, lane 2, and lane 3's *draft* step
(`release-testflight-notes.yml`) — and `DEEPL_AUTH_KEY` (lane 2). The iOS **publish** workflow no
longer calls the LLM: it reads the human-approved `whats-new.txt`, and with none present falls back
to the build-metadata note. Apple App Store note *upload* (`deliver`) is Stage 2, still deferred.

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
