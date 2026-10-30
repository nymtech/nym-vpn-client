# Release changelog automation

On a release-branch cut (or manual dispatch), `generate-release-changelog.yml` authors the
release changelogs from the milestone's merged PRs and opens **one PR** against the release
branch as the proofread gate. No agent runs in CI — each step is a stateless HTTPS call.

```
milestone PRs ─(a)author_changelog.py→ component CHANGELOG [Unreleased] ─→ existing stamp/extract CI
                                              │
                                              └(b)author_store.py→ out/*.en.txt ─translate.py→ fastlane files
```

- **(a) `author_changelog.py`** — gathers the milestone's merged PRs, **routes each by changed
  file path** (labels are too inconsistent to rely on), and makes one Anthropic call per
  component to write Keep-a-Changelog bullets into each `CHANGELOG.md` `[Unreleased]`. The
  existing `add-changelog-release` (version stamp) and `prepare-release-notes`
  (`extract-release-notes` → GitHub notes + TestFlight) consume those sections unchanged.
- **(b) `author_store.py`** — **derives** app-store copy from the `[Unreleased]` bullets
  (Android ← android+core, Apple ← apple+core), selecting and rewriting the user-facing ones
  for consumer voice. One source of truth; no second PR query.
- **`translate.py`** — DeepL Pro into every locale in `locales.json` (German informal `du`);
  Android auto-fits to ≤500 chars by dropping the lowest-ranked bullet and re-translating
  (never fails the build).

**The one human step:** proofread + approve the PR before the beta ships. DeepL is round-1 and
the English is model-drafted — the review is the gate, so **never auto-merge**.

## Secrets

`ANTHROPIC_API_KEY`, `DEEPL_AUTH_KEY`. (Apple store-note *upload* on public release is a separate
piece — a fastlane `deliver` lane + `publish-nym-vpn-apple-metadata.yml`; see the palace work
order. The ASC secrets already exist: `APP_STORE_CONNECT_API_KEY` / `_ISSUER_ID` / `_PRIVATE_KEY`.)

## Run locally

```sh
export MILESTONE='v2026.15 - Yari'
python scripts/release-changelog/author_changelog.py   # writes component CHANGELOG [Unreleased]
python scripts/release-changelog/author_store.py       # writes out/*.en.txt
python scripts/release-changelog/translate.py          # writes the fastlane files
```

Add `--dry-run` (print, write nothing) or `--no-llm` (skip the API — plumbing smoke test only)
to `author_changelog.py` / `author_store.py`.

## Notes / dependencies

- **Milestone tagging is load-bearing:** (a) authors from the milestone's merged PRs, so a PR not
  tagged to the milestone won't appear. Pass `MILESTONE` explicitly on dispatch when unsure.
- **Routing is by changed path**, falling back to `affects:*` labels, else `core`. Cross-platform
  features fan out to each platform's changelog (intended).
- Targets today's **four** component changelogs; if they're unified into one (separate work order),
  (a)'s write target adapts.
