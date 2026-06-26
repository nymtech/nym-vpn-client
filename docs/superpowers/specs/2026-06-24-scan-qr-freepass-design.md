# Design: Scan QR code to redeem a free-pass (Android)

Date: 2026-06-24
Status: Approved design — pending spec review

## Goal

Add a third button, **"Scan QR code"**, to the Android first-start (Welcome) screen.
Pressing it opens an in-app camera QR scanner. The scanner reads either:

- a full URL such as `https://nym.com/account/freepass?code=eJMWikx3EeU`, or
- a bare code such as `eJMWikx3EeU` (length varies).

On a successful scan the app:

1. creates a new account locally and registers it (existing flow), then
2. applies the scanned free-pass voucher code against the account.

On success the user continues to the post-account-creation screen (and can connect).
On failure (invalid or already-redeemed code) the user can try another code or go back.

## Key findings (current state of the codebase)

- **`apply_freepass` already exists in the Rust core** at
  `nym-vpn-core/crates/nym-vpn-api-client/src/client.rs:1036`
  (`ApplyFreepassRequestBody { code }`, `FREEPASS` route, `VpnApiClientError::ApplyFreepass`).
  It is **not** wired up through the account-controller → vpn-service → uniffi layers yet.
- The free-pass auth **Bearer JWT is minted inside Rust** from the account's secp256k1
  wallet (`nym-vpn-api-client/src/jwt.rs`, `pub(crate)`). The Android app therefore
  **cannot** call `nym.com/.../freepass` directly — it has no way to produce the token.
  The only clean path is to expose the existing Rust function through uniffi.
- **Scanner scaffolding already present:** `zxing-android-embedded` 4.3.0 is in
  `gradle/libs.versions.toml` and `app/build.gradle.kts:252`; `android.permission.CAMERA`
  and `com.journeyapps.barcodescanner.CaptureActivity` are already declared in
  `app/src/main/AndroidManifest.xml`. There is **no scan-launch code yet**.
- `accompanist-permissions` 0.37.3 is available in the catalog (for the runtime CAMERA
  permission prompt). CameraX is **not** in the catalog.
- Native build is fully scriptable: `nym-vpn-core/Android.mk` runs
  `cargo ndk build` + `uniffi-bindgen generate` + `strip`. NDK is at
  `~/Android/Sdk/ndk/27.1.12297006`, `cargo-ndk` 4.1.2 and the android Rust targets are
  installed, and `libwg.so` already exists (no wireguard-go rebuild needed).

## Decisions (confirmed with user)

| Decision | Choice |
|---|---|
| QR library | `zxing-android-embedded` (Apache-2.0, F-Droid compatible) — already in project |
| Build scope | Full end-to-end **including** the native `.so` + uniffi-binding rebuild |
| Success destination | Same as account-create flow (TechOpt if unseen, then Main) — but **skip SelectPlan** |
| Failure cleanup | **Keep** the created account; offer "Try another code" that re-applies only |
| Scanner UI | **Custom Compose scanner screen** (reusing zxing's camera/decoder, branded chrome) |
| Manual entry | Scanner screen also offers a text field to type/paste a code or URL |
| Auto-continue | A *valid* detected QR proceeds automatically — no extra tap needed |
| Code format | Base58 `[1-9A-HJ-NP-Za-km-z]`, length 4–128 |
| URL trust | Only `https` URLs whose host is `nym.com` or a `*.nym.com` subdomain |

## Architecture — three layers

### Layer 1 — Rust core: expose `apply_freepass` through uniffi

Follow the existing `create_account` wiring pattern exactly.

1. `nym-vpn-account-controller/src/commands/dispatch.rs`
   - Add variant `ApplyFreepass(ReturnSender<(), AccountCommandError>, String)` to
     `enum AccountCommand`.
   - Add the matching arm in `AccountCommand::return_error`.
2. `nym-vpn-account-controller` (handler + `command_sender.rs`)
   - Add `apply_freepass(code)` to `command_sender.rs` mirroring `create_account_command`.
   - Add a handler that loads the stored account and calls
     `vpn_api_client.apply_freepass(&account, code)`, then maps the result/error.
3. `nym-vpn-lib/src/service/vpn_service.rs`
   - Add `VpnServiceCommand::ApplyFreepass` and a `handle_apply_freepass` that forwards to
     the account command (mirroring `handle_create_account`).
4. `nym-vpn-lib-uniffi` (`vpn_service_command_sender.rs`, `vpn_account_storage.rs`,
   `account.rs`)
   - Expose `pub async fn apply_freepass(&self, code: String) -> Result<(), VpnError>`.
5. Error mapping — define distinguishable error states so the UI can tell the two failure
   cases apart:
   - `FreepassCodeInvalid` (code not recognized / malformed),
   - `FreepassCodeAlreadyRedeemed`,
   - generic `ApplyFreepassFailed` fallback (network/other).

   **⚠ Open verification:** the exact HTTP status/body the API returns for invalid vs
   already-redeemed must be confirmed by exercising the endpoint with the known codes
   (`eJMWikx3EeU` = valid, `hkB4sgMgfU8` = already redeemed). If the two cases are not
   distinguishable from the response, collapse to a single generic
   "couldn't apply this code" message. This is resolved during implementation/build, not
   guessed.

### Layer 2 — Native build + binding regen

From `nym-vpn-core/`:

```
make -f Android.mk build uniffi strip
```

This:
- cross-compiles `nym-vpn-lib-uniffi` + `nym-vpn-lib-types` for `arm64-v8a` and `x86_64`,
- regenerates the committed Kotlin bindings at
  `nym-vpn-android/core/src/main/java/net/nymtech/vpn/nym_vpn_lib/nym_vpn_lib_uniffi.kt`
  (and `nym_vpn_lib_types`),
- updates the committed `.so` files under
  `nym-vpn-android/core/src/main/jniLibs/{arm64-v8a,x86_64}/`.

After regen, the new `applyFreepass` binding is callable from Kotlin.

### Layer 3 — Android app

**Backend manager**
- `app/.../manager/backend/BackendManager.kt`: add
  `suspend fun applyFreepass(code: String)`.
- `app/.../manager/backend/ServiceBackedBackendManager.kt`: implement by calling the new
  uniffi binding; surface the typed error.

**Code parsing + validation helper (security boundary)**
- A single pure function `parseFreepassCode(raw: String): FreepassParseResult` is the
  **only** path any scanned or typed value takes before it reaches the backend. Both the
  camera decoder and the manual text field funnel through it. Returns a sealed result:
  `Valid(code)` or `Invalid(reason)` — never a raw passthrough.
- Algorithm:
  1. Trim surrounding whitespace. Reject if it contains any control characters or internal
     whitespace, or if length is absurd (> 4096 before parsing) — fail fast.
  2. If the value parses as a hierarchical URI **with a scheme**:
     - Require `scheme == "https"`. Reject otherwise (blocks `javascript:`, `file:`,
       `data:`, `intent:`, etc.).
     - Require host to equal `nym.com` or end with `.nym.com` (case-insensitive,
       exact-label match so `nym.com.evil.com` is rejected). Reject otherwise.
     - Read the `code` query parameter; if absent, `Invalid`.
     - The extracted `code` then goes through step 3.
  3. Validate the candidate code against the **allow-list** regex
     `^[1-9A-HJ-NP-Za-km-z]{4,128}$` (base58, length 4–128). Pass → `Valid(code)`,
     fail → `Invalid`.
- Examples:
  - `https://nym.com/account/freepass?code=eJMWikx3EeU` → `Valid("eJMWikx3EeU")`
  - `https://sub.nym.com/x?code=eJMWikx3EeU` → `Valid("eJMWikx3EeU")`
  - `eJMWikx3EeU` → `Valid("eJMWikx3EeU")`
  - `https://evil.com/?code=eJMWikx3EeU` → `Invalid` (untrusted host)
  - `javascript:alert(1)` / `'; DROP TABLE …` / 10 kB blob → `Invalid`
- Because validation is allow-list (only base58 passes) rather than deny-list, no crafted
  string can slip through to the backend. The code is additionally serialized as a JSON
  string field in Rust (`serde_json`), so there is no string-injection surface at the HTTP
  layer either; this helper is defense-in-depth and a UX gate.
- Unit-tested in isolation (see Testing).

**Welcome screen**
- `app/.../ui/screens/auth/components/WelcomeView.kt`: add a third `MainStyledButton`
  labeled "Scan QR code" with a QR-code leading icon, below the Login button.
- New string resource (e.g. `auth_scan_qr_button`) and a QR vector drawable.
- `WelcomeView` gains an `onScanQrClick: () -> Unit` parameter.

**Navigation**
- `app/.../ui/Route.kt`:
  - Add `data object FreepassScanner : Route()`.
  - Extend `Generating` to `data class Generating(val mode: String = ..., val code: String? = null)`.
- `app/.../ui/screens/account/generating/GeneratingScreen.kt`:
  `enum class GeneratingMode { CreateAccount, DeepLinkLogin, Freepass }`.
- `app/.../ui/screens/auth/AuthComponent.kt`: wire `WelcomeView.onScanQrClick` to
  `onAuthSuccess(); rootNavController.navigate(Route.FreepassScanner)`.
- `app/.../ui/MainActivity.kt`: register `composable<Route.FreepassScanner> { FreepassScannerScreen() }`.

**Scanner screen (new, custom Compose)**
- New package `app/.../ui/screens/account/scanner/` with `FreepassScannerScreen.kt`.
- Uses `accompanist-permissions` to request CAMERA at runtime; shows a rationale + a
  settings deep-link if permanently denied.
- Embeds zxing's `BarcodeView`/`DecoratedBarcodeView` (from the already-bundled
  `zxing-android-embedded`) inside an `AndroidView`, restricted to QR_CODE, wrapped in the
  app's own Compose chrome (title, instructions, back button) for a branded look. This
  satisfies "custom Compose scanner" without adding CameraX dependencies.
  - *Alternative if a fully Compose-native camera preview is preferred:* add the four
    `androidx.camera:*` artifacts and decode `ImageAnalysis` frames with
    `com.google.zxing` core. Noted but not the default.
- **Auto-continue on a valid scan:** each decoded payload is passed to
  `parseFreepassCode`. On the **first `Valid(code)`**, stop scanning and
  `rootNavController.navigate(Route.Generating(mode = Freepass, code = code))`
  (replacing the scanner in the back stack so Back returns to Welcome). No confirmation
  tap is required.
  - Decodes that return `Invalid` are **ignored** (the camera keeps scanning) so a stray
    or hostile QR cannot push the user forward. To avoid a silent dead-end, if only
    invalid codes are seen for a few seconds, show a non-blocking hint
    ("That doesn't look like a Nym free-pass code — try the code below").
- **Manual entry:** the screen includes an "Enter code manually" affordance that reveals a
  text field (single line, no multiline, sensible `maxLength`). The user types or pastes a
  code or a `nym.com` URL and taps Submit. The input goes through the **same**
  `parseFreepassCode`:
  - `Valid(code)` → navigate to `Route.Generating(Freepass, code)` exactly as the scan
    path does.
  - `Invalid` → inline field error ("Enter a valid free-pass code") and stay; nothing is
    sent.
- A back button returns to the Welcome/Main screen.

**Generating flow (Freepass mode)**
- `GeneratingViewModel`:
  - Read `code` from the route.
  - Logic: `if (!backendManager.isMnemonicStored()) backendManager.createAccount()`
    then `backendManager.applyFreepass(code)`.
    - Using `isMnemonicStored()` makes the **retry-apply-only** path automatic: on a
      retry the account already exists, so only `applyFreepass` runs again.
  - **Success** → set `pendingNavigation` to `TechOpt`-or-`Main` (mirroring the
    `!billingAvailable` branch of `CreateAccount`); **never** `SelectPlan` (the user
    already redeemed a pass).
  - **Failure** → emit a freepass-error event carrying the error kind (invalid /
    already-redeemed / generic). Do **not** forget the account.
- `GeneratingScreen`:
  - Reuse the existing animation gating for `Freepass` (treat like `CreateAccount`).
  - On freepass error, render an error state (or modal) with the appropriate message and
    two actions:
    - **Try another code** → `navigate(Route.FreepassScanner)` (re-scan; next Generating
      run skips create and only applies).
    - **Back to start** → `navigateAndForget(Route.Main(authRoute = Welcome))`.

## Data flow (happy path)

```
WelcomeView ─onScanQrClick─▶ FreepassScannerScreen
   │                              │ scan OR manual entry
   │                              │   → parseFreepassCode (allow-list)
   ▼                              ▼ Valid(code) only
(leave auth nav)        Route.Generating(Freepass, code)
                                  │
                          GeneratingViewModel
                          createAccount() [if not stored]
                          applyFreepass(code)
                                  │  ok
                                  ▼
                    TechOpt (if unseen) else Main ─▶ user can connect
```

Failure: `applyFreepass(code)` errors → GeneratingScreen error state →
{Try another code → FreepassScannerScreen} | {Back → Welcome}.

## Testing

- **Rust:** unit/integration coverage for the new account-controller command and error
  mapping where the existing controller tests live; verify against the real endpoint with
  `eJMWikx3EeU` (valid) and `hkB4sgMgfU8` (already redeemed) to lock down error mapping.
- **Kotlin:** exhaustively unit-test `parseFreepassCode` — bare valid code; trusted
  `nym.com` / `*.nym.com` URL; subdomain; bare-code with surrounding whitespace; and the
  rejection cases: untrusted host (`evil.com`, `nym.com.evil.com`), non-https scheme
  (`javascript:`, `file:`, `data:`, `intent:`), missing `code` param, non-base58 chars
  (`0`, `O`, `I`, `l`, symbols), too short / too long, internal whitespace, control
  characters, and oversized blobs.
- **Manual:** (1) scan a valid code → account created + pass applied + lands on connect;
  (2) scan the already-redeemed code → "already redeemed" message + Try-another-code works
  without creating a second account; (3) deny camera permission → rationale path then
  manual entry still works; (4) type a valid code manually → same success path; (5) point
  the camera at a non-Nym / malicious QR → ignored, hint shown, no navigation.

## Out of scope

- iOS / desktop clients.
- Any change to how accounts are created/registered beyond reusing existing calls.
- Replacing the QR library or the scanner with CameraX (documented as an alternative only).
