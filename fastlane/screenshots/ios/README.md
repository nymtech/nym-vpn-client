# iOS App Store screenshots (fastlane deliver)

Complete current screenshot sets for all **16 App Store localizations**, one folder per
App Store Connect locale. Uploaded by `fastlane deliver` (`screenshots_path` in `../../Deliverfile`).

- **iPhone** — 6.5" display, 1242×2688: `iphone_1.jpg` … `iphone_9.jpg`
- **iPad** — 13" display, 2752×2064 (landscape): `ipad_1.jpg` … `ipad_9.jpg`

`deliver` groups images into display families by resolution and orders each family by filename.

**Format: JPEG (quality 90).** App Store Connect accepts JPEG for screenshots, and at q90
these are visually indistinguishable on-device from the lossless source (the dark gradient
hero frame — the worst case for banding — is clean). This holds the tree at ~127 MB instead
of ~223 MB as lossless PNG+alpha; the pixel dimensions are Apple's mandated sizes and cannot
be reduced. The source PNGs were the Figma export — converting once from that lossless source
to q90 is equivalent to exporting JPEG directly.

## Provenance

Re-exported fresh from Figma **"V2 App Store Images"** (file `oTGHYAfPRZ60X76EWPPDDm`) on
**2026-11-04** — the current **post-dVPN** frames (mode labels read "dVPN", not the old
"Fast"). This supersedes the June export (pre-dVPN, stale) and the partial Sept on-disk export.

Locale mapping from the Figma frame names: `ES-MX → es-ES`, `ZH → zh-Hans`, `JP → ja`;
the rest map directly. 288 files total (16 × 2 devices × 9), all dimension-verified.

## Caution

`overwrite_screenshots` is **enabled** in the `Deliverfile`, so a run clears the existing
App Store screenshots before uploading these. Verify the first prod run (dry run / preview)
before trusting it — confirm it doesn't drop a display family not represented here.
