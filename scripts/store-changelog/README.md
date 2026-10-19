# Store changelog automation

Generates the per-locale **store** release notes for a NymVPN release and opens a PR:

- **Android** → `fastlane/metadata/android/<locale>/changelogs/<versionCode>.txt` (uploaded by `fastlane supply`, Play Store limit 500 chars)
- **Apple** → `nym-vpn-apple/fastlane/metadata/<locale>/release_notes.txt` (uploaded by `fastlane deliver`, see `publish-nym-vpn-apple-metadata.yml`)

These are the **public store** notes — distinct from the component `CHANGELOG.md` files (which feed the GitHub release notes) and from TestFlight's "what to test" notes.

## How it runs

No agent/skill runs in CI. The workflow `generate-store-changelog.yml` runs on the CI runner and makes stateless HTTPS calls:

1. `author.py` — `gh` lists the milestone's merged PRs, then **one Anthropic Messages API call** drafts ranked, user-facing English copy for Android + Apple.
2. `translate.py` — **DeepL Pro** translates into every locale in `locales.json`; German uses informal `du`. Android auto-fits to <=500 chars by dropping the lowest-ranked bullet and re-translating (it never fails the build).
3. The workflow opens a PR labelled `needs-proofread` against the release branch.

**The one human step:** proofread + approve that PR before the beta ships. DeepL is round-1 and the English is model-drafted, so the review is the gate — never auto-merge.

## Secrets required

- `ANTHROPIC_API_KEY`
- `DEEPL_AUTH_KEY`

## Config

`locales.json` maps each locale to its DeepL code, Android fastlane dir, Apple fastlane dir
(`null` where the App Store has no such localization, e.g. Persian/Bengali), and per-locale
formality. Edit it to add/remove locales or change formality policy.

## Run locally

```sh
export MILESTONE='v2026.14 - Xaubub' ANTHROPIC_API_KEY=... DEEPL_AUTH_KEY=... GH_TOKEN=...
python scripts/store-changelog/author.py       # writes out/*.en.txt (review/edit these)
python scripts/store-changelog/translate.py    # writes the fastlane files
```

`out/*.en.txt` is the English source of truth — edit it and re-run `translate.py` to regenerate.
