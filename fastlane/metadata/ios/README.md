# iOS App Store metadata (fastlane deliver)

`fastlane deliver` uploads the App Store **listing text** from this tree to App Store
Connect — one folder per App Store localization:

    fastlane/metadata/ios/<locale>/
      name.txt              # ≤ 30 code points
      subtitle.txt          # ≤ 30
      keywords.txt          # ≤ 100 (comma-separated, no spaces)
      promotional_text.txt  # ≤ 170
      description.txt       # ≤ 4000

## What's here

The **16 App Store localizations**, 5 fields each (80 files), generated from the authoritative
source of truth: `metadata.json` in `Deliverables/NymVPN app store metadata/App Store metadata
as of nymvpn v2026.14/`. All fields QA'd within Apple's limits. Descriptions correct the stale
live listings (pre-dVPN wording). No price line — App Store text is per-language not per-region,
so the purchase sheet shows the real regional price (decided 2026-11-04).

- `metadata_path` is pinned to `./fastlane/metadata/ios` so deliver never touches the Android
  `supply` metadata under `fastlane/metadata/android`.
- `ar-SA` keywords were captured from live App Store Connect (2026-11-04).

## Not managed here (by design)

- **Screenshots** — now present under `fastlane/screenshots/ios/<locale>/` (added 2026-11-04, re-exported from Figma; see that folder's README).
- **`release_notes` (What's New)** — version-specific; sourced at release time from the changelog flow.
- **support/marketing/privacy URLs, copyright, categories, review info (demo account)** — stable,
  left App-Store-Connect-managed; deliver only touches fields whose files exist here.
