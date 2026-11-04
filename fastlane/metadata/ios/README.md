# iOS App Store metadata (fastlane deliver)

`fastlane deliver` uploads the App Store **listing text** from this tree to App Store
Connect — one folder per App Store localization, e.g.:

    fastlane/metadata/ios/en-US/
      name.txt              # ≤ 30 code points
      subtitle.txt          # ≤ 30
      keywords.txt          # ≤ 100 (comma-separated, no spaces)
      promotional_text.txt  # ≤ 170
      description.txt       # ≤ 4000

## Empty on purpose

The pipeline is wired and statically verifiable — `Appfile` (`for_platform :ios`),
the `:ios upload_metadata` lane in `Fastfile`, `Deliverfile`, and
`.github/workflows/publish-nym-vpn-ios-metadata.yml` — but the **localized copy is
pending sign-off** and lands in a follow-up commit. Until copy is added here, the
lane uploads nothing (deliver finds no locale folders).

- `metadata_path` is pinned to `./fastlane/metadata/ios` so deliver never touches the
  Android `supply` metadata under `fastlane/metadata/android`.
- Screenshots are deferred; when added they go under `fastlane/screenshots/ios/<locale>/`
  and `skip_screenshots` / `overwrite_screenshots` in the `Deliverfile` get flipped then.
- Submission is metadata-only: a human submits the version for review.
