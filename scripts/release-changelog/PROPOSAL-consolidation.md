# Proposal: one process for changelogs, release notes & store copy (Stage 1)

**Status:** draft, awaiting sign-off (Casey). No code changes until approved.
**Branch:** `casey/changelog-automation`. **Repo:** `nymtech/nym-vpn-client`.

> `nym-vpn-client` has no OpenSpec/`AGENTS.md`; this doc is the equivalent proposal + spec
> deltas + tasks, kept with the code so it rides in the PR for review.

## Why

Changelog / release-note / store-copy handling is spread across several workflows. The goal is
**one process: one producer, one stamper, one compiler, one store lane, one guard — no redundant or
secondary workflow touching these files, and nothing left out.** Four gaps stand between us and that.

1. **A redundant second stamper.** `tauri-post-release.yml` edits `nym-vpn-app/CHANGELOG.md` on
   develop (inserts an empty `## [version]` header). A ~14-month leftover from when the tauri app
   shipped on its own cadence (created 2025-08-05, `#3210`); the unified stamp only absorbed the app
   changelog on 2026-08-04 (`#5992`) and this step was never removed.
2. **A narrow guard with a hole.** `check-changelog.yaml` checks only Play changelogs (≤500) and
   only on **develop** PRs — so no other store field is length-checked, and the release-branch PR
   that introduces the store copy isn't checked at all.
3. **Beta TestFlight notes are stubbed.** Betas publish to **TestFlight** (a public test asset), and
   `publish-nym-vpn-ios.yml` feeds it a fabricated `whatsnew.txt` (Date/Branch/commit) instead of
   real notes. **Android does not** upload store copy at betas (confirmed — Play alpha does not
   consume consumer changelogs here), so this is Apple-only.
4. **Apple App Store notes have no uploader (Stage 2).** Nothing pushes
   `nym-vpn-apple/fastlane/.../release_notes.txt` to App Store Connect; those files are orphaned on
   the production side.

## What is already correct (no change)

- **Producer** — `author-pr-changelog.yml`: PR `## Description` → component `CHANGELOG.md`
  `[Unreleased]`, milestone-gated, keyed by PR URL.
- **Compiler** — `prepare-release-notes.yml`: `[Unreleased]` (beta) / stamped `[version]` (release)
  → `release-notes.md`, consumed by `release-release-artifacts.yml` (`gh release ... --notes-file`).
- **Android upload path** — `publish-nym-vpn-android-metadata.yml`'s `fastlane supply` reads
  `fastlane/metadata/android/<loc>/changelogs/<versionCode>.txt` (filename = Android `VERSION_CODE`)
  and uploads to Google Play; `translate.py` already writes exactly those paths/names.

## Resolved decisions (from sign-off round 1)

- **TestFlight notes source = the LLM consumer store copy** (apple, `author_store.py`), not the
  compiled developer notes.
- **This moves the Apple consumer copy from final-only to per-beta** (TestFlight is a beta asset).
  **Android store copy stays final-only** — Android does not publish store metadata at betas.
- **Guard placement = both** — a CI check on PRs *and* an in-generator check inside
  `release-store-notes.yml`.
- **Guard scope = all fastlane fields**, not just changelogs (new requirement — change 2).

## Scope

- **Stage 1 (this proposal):** changes 1–3 below (the redundant stamper, the all-fields guard, and
  per-beta consumer store copy incl. the TestFlight wiring and the release-close ordering gate).
- **Stage 2 (deferred, confirmed in scope):** Apple App Store Connect note upload — add
  `fastlane deliver` consuming `release_notes.txt`; needs ASC API credentials.

## Changes (Stage 1)

### 1. Remove the changelog step from `tauri-post-release.yml`
Delete its `Update CHANGELOG.md` step and the `git add nym-vpn-app/CHANGELOG.md`; keep the
Linux-installer and flatpak steps. The app changelog is then stamped only by `add-changelog-release`.
**Pre-removal check (blocking):** confirm no standalone app-only release path calls
`tauri-post-release.yml` without a preceding `bump-all-platforms` stamp.

### 2. One fastlane-metadata length guard, all fields, develop + release
Replace `check-changelog.yaml` with a single comprehensive check (`check-fastlane-metadata.yaml` +
`scripts/release-changelog/check_fastlane_limits.py`) that validates **every** fastlane metadata
file against its platform limit, for any PR touching fastlane metadata on **develop or
`release/**`**, and is **reused inside `release-store-notes.yml`** before it opens its PR.

Limits (to re-verify against current store docs at implementation time):

| Platform | File | Limit (chars) |
|---|---|---|
| Google Play | `title.txt` | 30 |
| Google Play | `short_description.txt` | 80 |
| Google Play | `full_description.txt` | 4000 |
| Google Play | `changelogs/<code>.txt` | 500 |
| App Store | `name.txt` | 30 |
| App Store | `subtitle.txt` | 30 |
| App Store | `promotional_text.txt` | 170 |
| App Store | `keywords.txt` | 100 |
| App Store | `description.txt` | 4000 |
| App Store | `release_notes.txt` | 4000 |

### 3. Per-beta Apple TestFlight notes + release-close gate
- At each **beta**, generate the Apple consumer "what's new" (`author_store.py`, English) from the
  release's `[Unreleased]`, and point `publish-nym-vpn-ios.yml` `--whats-new` at it, with a safe
  fallback to today's stub when absent. **Auto-generated, no proofread PR** — betas stay smooth and
  testers get real notes. No translation and **no Android** at beta.
- The **full store lane stays at final** (`release-store-notes.yml`: both platforms, all locales,
  one needs-proofread PR) — unchanged except for the guard in change 2.
- **Release-close ordering gate** (proposed signal, per "propose based on current model"): the
  authoring workflow treats a milestone version *V* as **closed** when the base branch's component
  `CHANGELOG.md` already contains a **stamped** `## [V] - <date>` section (date ≠ `TBD`) — the exact
  state the final stamp produces — corroborated by an existing `nym-vpn-core-v<V>` tag/release. A PR
  milestoned to a closed *V* fails with guidance (otherwise it would silently author into the next
  `[Unreleased]`).

## Spec deltas (observable behaviour)

- **MODIFIED — app changelog stamping:** `nym-vpn-app/CHANGELOG.md` is stamped exactly once per
  release, by the unified rename stamper; no empty/duplicate `## [version]` headers on develop.
- **ADDED — full fastlane length guard:** a PR adding/modifying any fastlane metadata field on
  develop or `release/**` fails CI if that field exceeds its platform limit; the generator applies
  the same check before opening its PR.
- **MODIFIED — beta TestFlight notes:** TestFlight betas ship real Apple consumer notes derived from
  `[Unreleased]`, not a Date/Branch/commit stub. (Android store copy unchanged — final only.)
- **ADDED — release-close ordering:** a PR milestoned to an already-stamped release is failed rather
  than authoring into the next release's `[Unreleased]`.

## Tasks

- [ ] Verify no standalone app-only release path bypasses the unified stamp (gate for change 1).
- [ ] Remove the changelog step + `git add` from `tauri-post-release.yml`.
- [ ] Write `check_fastlane_limits.py` (limits table, android + apple) and
      `check-fastlane-metadata.yaml` (develop + `release/**`); retire `check-changelog.yaml`.
- [ ] Call the same length check inside `release-store-notes.yml` before PR creation.
- [ ] At beta, run `author_store` (apple, English) and point `publish-nym-vpn-ios.yml`
      `--whats-new` at it, with a fallback (Apple-only; no translation, no Android, no PR).
- [ ] Implement the release-close ordering gate (stamped-section + tag signal).
- [ ] Update `scripts/release-changelog/README.md` to the single consolidated process.
- [ ] Dry-run validation across cut → beta → final.

## Open decisions (for sign-off)

All round-1 decisions are now resolved and folded in. Remaining confirmations:

1. **Fastlane limit values** — confirm the table in change 2 against current Play/App Store docs at
   implementation time (limits occasionally change).
2. **Release-close gate signal** — confirm the stamped-`## [V]`-section (+ `nym-vpn-core-v<V>` tag)
   signal is acceptable as the "release closed" trigger.
