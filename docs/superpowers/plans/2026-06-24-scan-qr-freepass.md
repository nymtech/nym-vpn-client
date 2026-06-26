# Scan-QR Free-Pass Redemption — Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Add a "Scan QR code" button to the Android Welcome screen that scans (or accepts a manually typed) Nym free-pass code, creates+registers an account, applies the free-pass voucher, and continues to the connect screen.

**Architecture:** The free-pass HTTP call (`apply_freepass`) already exists in the Rust `nym-vpn-api-client`. We expose it up the existing command chain — account-controller → `nym-vpn-lib` vpn-service → `nym-vpn-lib-uniffi` `NymVpnServiceCommandSender` — then rebuild the native libs + regenerate the committed Kotlin uniffi bindings. The Android app gains a backend method, a validated code parser, a custom Compose scanner screen (camera + manual entry), and reuses the existing `Generating` screen to drive create→apply.

**Tech Stack:** Rust (tokio, uniffi), Android (Kotlin, Jetpack Compose, Hilt, `zxing-android-embedded` 4.3.0, `accompanist-permissions` 0.37.3), `cargo-ndk` + `uniffi-bindgen`.

## Global Constraints

- F-Droid compatible: no Google Play / proprietary deps. Use `zxing-android-embedded` (already in catalog) — never ML Kit.
- Code validation is **allow-list only**: `^[1-9A-HJ-NP-Za-km-z]{4,128}$` (base58, length 4–128). Every scanned or typed value passes through one parser before reaching the backend.
- Trusted URL hosts: `https` scheme AND host equals `nym.com` or ends with `.nym.com` (exact-label match).
- On apply failure: **keep** the created account (retry re-applies only; do not re-create, do not forget).
- Success destination mirrors account-create's non-billing branch (TechOpt if unseen, else Main) — **never** `SelectPlan`.
- Test free-pass codes: `eJMWikx3EeU` (valid), `hkB4sgMgfU8` (already redeemed).
- NDK at `~/Android/Sdk/ndk/27.1.12297006`; `libwg.so` already built (no wireguard-go rebuild).
- Rust files end SPDX header style already present; match each crate's existing import/format conventions. Run `cargo fmt` per touched crate.

---

### Task 1: Rust — account-controller `ApplyFreepass` command

Wire a new account command that calls the existing `VpnApiClient::apply_freepass`.

**Files:**
- Modify: `nym-vpn-core/crates/nym-vpn-account-controller/src/commands/dispatch.rs`
- Modify: `nym-vpn-core/crates/nym-vpn-account-controller/src/commands/handler.rs`
- Modify: `nym-vpn-core/crates/nym-vpn-account-controller/src/command_sender.rs`
- Modify (one new match arm each):
  - `.../src/state_machine/ready_state.rs`
  - `.../src/state_machine/logged_out_state.rs`
  - `.../src/state_machine/offline_state.rs`
  - `.../src/state_machine/error_state.rs`
  - `.../src/state_machine/pending_subscription_state.rs`
  - `.../src/state_machine/upgrade_mode_state.rs`
  - `.../src/state_machine/syncing_state/network_state.rs`
  - `.../src/state_machine/syncing_state/local_state.rs`
  - `.../src/state_machine/syncing_state/requesting_zknym_state.rs`

**Interfaces:**
- Consumes: `VpnApiClient::apply_freepass(&self, account: &VpnAccount, code: String) -> Result<NymVpnSubscription>` (exists, `client.rs:1036`); `AccountCommandError::{NoAccountStored, VpnApi}` (exists; `From<VpnApiClientError>` exists, `account/mod.rs:213`).
- Produces: `AccountCommandSender::apply_freepass(&self, code: String) -> Result<(), AccountCommandError>`; `AccountCommand::ApplyFreepass(ReturnSender<(), AccountCommandError>, String)`; `handler::handle_apply_freepass`.

- [ ] **Step 1: Add the command variant**

In `dispatch.rs`, add to `enum AccountCommand` (after `ObtainTicketbooks`):

```rust
    /// Apply a free-pass voucher code to the stored account
    ApplyFreepass(ReturnSender<(), AccountCommandError>, String),
```

And add the arm to `AccountCommand::return_error` (alongside the other top-level arms):

```rust
            AccountCommand::ApplyFreepass(return_sender, _) => return_sender.send(Err(error)),
```

- [ ] **Step 2: Add the handler**

In `handler.rs`, add (mirroring `handle_register_account`'s access to `shared_state.vpn_api_client`):

```rust
pub(crate) async fn handle_apply_freepass<C: ConnectivityMonitor>(
    shared_state: &mut SharedAccountState<C>,
    code: String,
) -> Result<(), AccountCommandError> {
    let account = shared_state
        .vpn_api_account
        .clone()
        .ok_or(AccountCommandError::NoAccountStored)?;
    shared_state
        .vpn_api_client
        .apply_freepass(account.as_ref(), code)
        .await?;
    tracing::debug!("Free-pass applied to account");
    Ok(())
}
```

> If `shared_state.vpn_api_account` is not `Option<Arc<VpnAccount>>`, adjust the deref to match (grep `vpn_api_account` in `handler.rs` — `handle_register_account` shows the field usage). `apply_freepass` consumes `code: String` and borrows the account.

- [ ] **Step 3: Add the sender method**

In `command_sender.rs`, add inside `impl AccountCommandSender` (mirror `create_account_command`):

```rust
    #[instrument(skip(self))]
    pub async fn apply_freepass(&self, code: String) -> Result<(), AccountCommandError> {
        let (tx, rx) = ReturnSender::new();
        self.command_tx
            .send(AccountCommand::ApplyFreepass(tx, code))
            .map_err(AccountCommandError::internal)?;
        rx.await.map_err(AccountCommandError::internal)?
    }
```

- [ ] **Step 4: Add the match arm in every state**

Each state's `match command { … }` over `AccountCommand` is exhaustive, so add this **identical** arm to all nine state files listed above. Place it next to the existing `AccountCommand::ObtainTicketbooks` / `AccountCommand::RotateKeys` arms:

```rust
                    AccountCommand::ApplyFreepass(return_sender, code) => {
                        let res = handler::handle_apply_freepass(shared_state, code).await;
                        return_sender.send(res);
                    }
```

Notes:
- In states that don't return `NextAccountControllerState` from inline arms but use a different shape, match the surrounding arms' return convention (e.g. some files wrap in `return_sender.send(...)` then fall through; copy the neighbour `RotateKeys` arm's shape exactly).
- In states where `shared_state` is borrowed differently (e.g. `&mut`), the call still compiles since the handler takes `&mut SharedAccountState`. If a state has no `shared_state` in scope (pure error states), return the error instead: `AccountCommand::ApplyFreepass(return_sender, _) => return_sender.send(Err(AccountCommandError::NoAccountStored)),` — check each file's existing `CreateAccount` arm to decide which form it uses.

- [ ] **Step 5: Compile**

Run: `cd nym-vpn-core && cargo check -p nym-vpn-account-controller`
Expected: PASS (no non-exhaustive-match errors). Then `cargo fmt -p nym-vpn-account-controller`.

- [ ] **Step 6: Commit**

```bash
git add nym-vpn-core/crates/nym-vpn-account-controller
git commit -m "feat(core): add ApplyFreepass account command"
```

---

### Task 2: Rust — `nym-vpn-lib` vpn-service command

Forward an `ApplyFreepass` service command to the account command sender.

**Files:**
- Modify: `nym-vpn-core/crates/nym-vpn-lib/src/service/vpn_service.rs`

**Interfaces:**
- Consumes: `AccountCommandSender::apply_freepass` (Task 1); `self.account_command_tx`.
- Produces: `VpnServiceCommand::ApplyFreepass(oneshot::Sender<Result<(), AccountCommandError>>, String)`; `handle_apply_freepass`.

- [ ] **Step 1: Add the enum variant**

In the `VpnServiceCommand` enum (near `CreateAccount`, ~line 143):

```rust
    ApplyFreepass(oneshot::Sender<Result<(), AccountCommandError>>, String),
```

- [ ] **Step 2: Add the dispatch arm**

In the command-handling `match` (near the `VpnServiceCommand::CreateAccount` arm, ~line 1084):

```rust
            VpnServiceCommand::ApplyFreepass(tx, code) => {
                let _ = tx.send(self.handle_apply_freepass(code).await);
            }
```

- [ ] **Step 3: Add the handler method**

Next to `handle_create_account` (~line 1863):

```rust
    async fn handle_apply_freepass(&self, code: String) -> Result<(), AccountCommandError> {
        self.account_command_tx.apply_freepass(code).await
    }
```

- [ ] **Step 4: Compile**

Run: `cd nym-vpn-core && cargo check -p nym-vpn-lib`
Expected: PASS. Then `cargo fmt -p nym-vpn-lib`.

- [ ] **Step 5: Commit**

```bash
git add nym-vpn-core/crates/nym-vpn-lib
git commit -m "feat(core): route ApplyFreepass through vpn-service"
```

---

### Task 3: Rust — uniffi `NymVpnServiceCommandSender::apply_freepass`

Expose the call to Kotlin via the uniffi-exported command sender.

**Files:**
- Modify: `nym-vpn-core/crates/nym-vpn-lib-uniffi/src/vpn_service_command_sender.rs`

**Interfaces:**
- Consumes: `VpnServiceCommand::ApplyFreepass` (Task 2); `NymVpnServiceCommandInnerError::Account`.
- Produces: uniffi method `apply_freepass(code: String) -> Result<()>` → generated Kotlin `NymVpnServiceCommandSender.applyFreepass(code: String)`.

- [ ] **Step 1: Add the method**

In `vpn_service_command_sender.rs`, inside the `#[uniffi::export(async_runtime = "tokio")] impl NymVpnServiceCommandSender` block, mirror `create_account`:

```rust
    pub async fn apply_freepass(&self, code: String) -> Result<()> {
        self.send_and_wait(VpnServiceCommand::ApplyFreepass, code)
            .await?
            .map_err(NymVpnServiceCommandInnerError::Account)?;
        Ok(())
    }
```

> Verify `send_and_wait`'s generic accepts `code: String` as the payload type the same way `StoreAccount` accepts a request. If `send_and_wait` requires the payload as the second arg, this matches the `store_account` shape (`VpnServiceCommand::StoreAccount, request`).

- [ ] **Step 2: Compile (host target)**

Run: `cd nym-vpn-core && cargo check -p nym-vpn-lib-uniffi`
Expected: PASS. Then `cargo fmt -p nym-vpn-lib-uniffi`.

- [ ] **Step 3: Commit**

```bash
git add nym-vpn-core/crates/nym-vpn-lib-uniffi
git commit -m "feat(core): expose applyFreepass via uniffi command sender"
```

---

### Task 4: Native build + binding regeneration

Cross-compile the libs and regenerate the committed Kotlin bindings.

**Files:**
- Regenerated: `nym-vpn-android/core/src/main/java/net/nymtech/vpn/nym_vpn_lib/nym_vpn_lib_uniffi.kt`
- Rebuilt: `nym-vpn-android/core/src/main/jniLibs/{arm64-v8a,x86_64}/libnym_vpn_lib.so`, `libnym_vpn_lib_types.so`

- [ ] **Step 1: Build + generate + strip**

```bash
cd nym-vpn-core
export ANDROID_NDK_HOME=$HOME/Android/Sdk/ndk/27.1.12297006
make -f Android.mk build uniffi strip
```

Expected: builds `arm64-v8a` + `x86_64`, regenerates `nym_vpn_lib_uniffi.kt`, strips `.so`s. (Several minutes.)

- [ ] **Step 2: Verify the binding exists**

Run: `grep -n "fun \`applyFreepass\`" nym-vpn-android/core/src/main/java/net/nymtech/vpn/nym_vpn_lib/nym_vpn_lib_uniffi.kt`
Expected: a `fun \`applyFreepass\`(\`code\`: kotlin.String)` on `NymVpnServiceCommandSenderInterface` / `NymVpnServiceCommandSender`.

- [ ] **Step 3: Commit**

```bash
git add nym-vpn-android/core/src/main/java/net/nymtech/vpn/nym_vpn_lib/nym_vpn_lib_uniffi.kt nym-vpn-android/core/src/main/jniLibs
git commit -m "build(android): regenerate uniffi bindings with applyFreepass"
```

---

### Task 5: Android — backend `applyFreepass`

Thread the binding through the Android backend layers.

**Files:**
- Modify: `nym-vpn-android/core/src/main/java/net/nymtech/vpn/backend/api/VpnServiceApi.kt`
- Modify: `nym-vpn-android/core/src/main/java/net/nymtech/vpn/backend/api/VpnServiceApiImpl.kt`
- Modify: `nym-vpn-android/app/src/main/java/net/nymtech/nymvpn/manager/backend/BackendManager.kt`
- Modify: `nym-vpn-android/app/src/main/java/net/nymtech/nymvpn/manager/backend/ServiceBackedBackendManager.kt`

**Interfaces:**
- Consumes: `NymVpnServiceCommandSender.applyFreepass(code)` (Task 4); `core.requireCoreSender { … }`.
- Produces: `BackendManager.applyFreepass(code: String)` (suspend); `VpnServiceApi.applyFreepass(code: String)`.

- [ ] **Step 1: Add to `VpnServiceApi` interface**

In `VpnServiceApi.kt`, after `suspend fun createAccount()`:

```kotlin
	suspend fun applyFreepass(code: String)
```

- [ ] **Step 2: Implement in `VpnServiceApiImpl`**

In `VpnServiceApiImpl.kt`, after the `createAccount()` override:

```kotlin
	override suspend fun applyFreepass(code: String) {
		Timber.tag(TAG).d("applyFreepass requested")
		core.requireCoreSender { it.applyFreepass(code) }
	}
```

- [ ] **Step 3: Add to `BackendManager` interface**

In `BackendManager.kt`, after `suspend fun createAccount()`:

```kotlin
	suspend fun applyFreepass(code: String)
```

- [ ] **Step 4: Implement in `ServiceBackedBackendManager`**

After the `createAccount()` override (~line 214):

```kotlin
	override suspend fun applyFreepass(code: String) {
		serviceConnectionManager.withApi { it.applyFreepass(code) }
	}
```

- [ ] **Step 5: Compile**

Run: `cd nym-vpn-android && ./gradlew :core:compileGeneralDebugKotlin :app:compileGeneralDebugKotlin`
Expected: PASS.

- [ ] **Step 6: Commit**

```bash
git add nym-vpn-android/core nym-vpn-android/app
git commit -m "feat(android): add applyFreepass to backend manager"
```

---

### Task 6: Android — `parseFreepassCode` validator (TDD)

The single security gate. Pure function, fully unit-tested.

**Files:**
- Create: `nym-vpn-android/app/src/main/java/net/nymtech/nymvpn/util/FreepassCode.kt`
- Test: `nym-vpn-android/app/src/test/java/net/nymtech/nymvpn/util/FreepassCodeTest.kt`

**Interfaces:**
- Produces: `sealed interface FreepassParseResult { data class Valid(val code: String); data object Invalid }`; `fun parseFreepassCode(raw: String): FreepassParseResult`.

- [ ] **Step 1: Write the failing test**

```kotlin
package net.nymtech.nymvpn.util

import org.junit.Assert.assertEquals
import org.junit.Test

class FreepassCodeTest {
	private fun valid(s: String) = parseFreepassCode(s) as? FreepassParseResult.Valid

	@Test fun bareCode() = assertEquals("eJMWikx3EeU", valid("eJMWikx3EeU")?.code)
	@Test fun bareCodeTrimmed() = assertEquals("eJMWikx3EeU", valid("  eJMWikx3EeU \n")?.code)
	@Test fun trustedUrl() = assertEquals("eJMWikx3EeU", valid("https://nym.com/account/freepass?code=eJMWikx3EeU")?.code)
	@Test fun trustedSubdomain() = assertEquals("eJMWikx3EeU", valid("https://sub.nym.com/x?code=eJMWikx3EeU")?.code)
	@Test fun trustedUrlHostCaseInsensitive() = assertEquals("eJMWikx3EeU", valid("https://NYM.com/?code=eJMWikx3EeU")?.code)

	@Test fun rejectsUntrustedHost() = assertEquals(FreepassParseResult.Invalid, parseFreepassCode("https://evil.com/?code=eJMWikx3EeU"))
	@Test fun rejectsLookalikeHost() = assertEquals(FreepassParseResult.Invalid, parseFreepassCode("https://nym.com.evil.com/?code=eJMWikx3EeU"))
	@Test fun rejectsHttpScheme() = assertEquals(FreepassParseResult.Invalid, parseFreepassCode("http://nym.com/?code=eJMWikx3EeU"))
	@Test fun rejectsJavascriptScheme() = assertEquals(FreepassParseResult.Invalid, parseFreepassCode("javascript:alert(1)"))
	@Test fun rejectsFileScheme() = assertEquals(FreepassParseResult.Invalid, parseFreepassCode("file:///etc/passwd"))
	@Test fun rejectsMissingCodeParam() = assertEquals(FreepassParseResult.Invalid, parseFreepassCode("https://nym.com/account/freepass"))
	@Test fun rejectsNonBase58() = assertEquals(FreepassParseResult.Invalid, parseFreepassCode("abc0OIl"))
	@Test fun rejectsSymbols() = assertEquals(FreepassParseResult.Invalid, parseFreepassCode("abc';DROP"))
	@Test fun rejectsTooShort() = assertEquals(FreepassParseResult.Invalid, parseFreepassCode("ab"))
	@Test fun rejectsTooLong() = assertEquals(FreepassParseResult.Invalid, parseFreepassCode("a".repeat(129)))
	@Test fun rejectsInternalWhitespace() = assertEquals(FreepassParseResult.Invalid, parseFreepassCode("eJMW ikx3EeU"))
	@Test fun rejectsControlChars() = assertEquals(FreepassParseResult.Invalid, parseFreepassCode("eJMW ikx3"))
	@Test fun rejectsOversizedBlob() = assertEquals(FreepassParseResult.Invalid, parseFreepassCode("x".repeat(5000)))
	@Test fun rejectsEmpty() = assertEquals(FreepassParseResult.Invalid, parseFreepassCode(""))
}
```

- [ ] **Step 2: Run test to verify it fails**

Run: `cd nym-vpn-android && ./gradlew :app:testGeneralDebugUnitTest --tests "*FreepassCodeTest*"`
Expected: FAIL — `parseFreepassCode` unresolved.

- [ ] **Step 3: Implement**

```kotlin
package net.nymtech.nymvpn.util

import android.net.Uri

sealed interface FreepassParseResult {
	data class Valid(val code: String) : FreepassParseResult
	data object Invalid : FreepassParseResult
}

private val BASE58_CODE = Regex("^[1-9A-HJ-NP-Za-km-z]{4,128}$")
private const val MAX_RAW_LEN = 4096

private fun isTrustedHost(host: String?): Boolean {
	if (host == null) return false
	val h = host.lowercase()
	return h == "nym.com" || h.endsWith(".nym.com")
}

private fun validateCode(candidate: String): FreepassParseResult =
	if (BASE58_CODE.matches(candidate)) FreepassParseResult.Valid(candidate) else FreepassParseResult.Invalid

fun parseFreepassCode(raw: String): FreepassParseResult {
	val trimmed = raw.trim()
	if (trimmed.isEmpty() || trimmed.length > MAX_RAW_LEN) return FreepassParseResult.Invalid
	if (trimmed.any { it.isWhitespace() || it.isISOControl() }) return FreepassParseResult.Invalid

	// Looks like a URL? (has a scheme separator) — must be a trusted https nym.com URL.
	if (trimmed.contains("://") || trimmed.substringBefore(':', "").let { it.isNotEmpty() && it.all { c -> c.isLetter() } && trimmed.contains(":") && !BASE58_CODE.matches(trimmed) && trimmed.contains("/") }) {
		val uri = runCatching { Uri.parse(trimmed) }.getOrNull() ?: return FreepassParseResult.Invalid
		if (!uri.scheme.equals("https", ignoreCase = true)) return FreepassParseResult.Invalid
		if (!isTrustedHost(uri.host)) return FreepassParseResult.Invalid
		val code = uri.getQueryParameter("code") ?: return FreepassParseResult.Invalid
		return validateCode(code)
	}

	return validateCode(trimmed)
}
```

> `Uri.parse` is an Android framework call; the unit test runs on the JVM. If the project's `testOptions` does not set `unitTests.isReturnDefaultValues`/Robolectric, replace `Uri` usage with `java.net.URI` (`URI(trimmed).let { it.scheme; it.host; query parse }`) so the test runs without an emulator. Check `app/build.gradle.kts testOptions` first; prefer `java.net.URI` + manual query-param split to keep the test a pure JVM test. Re-confirm all test cases pass with whichever API is used.

- [ ] **Step 4: Run test to verify it passes**

Run: `cd nym-vpn-android && ./gradlew :app:testGeneralDebugUnitTest --tests "*FreepassCodeTest*"`
Expected: PASS (all cases). Simplify the URL-detection condition in step 3 if any case fails — the intent is: treat as URL iff it parses with a non-empty scheme; otherwise validate as a bare code.

- [ ] **Step 5: Commit**

```bash
git add nym-vpn-android/app/src/main/java/net/nymtech/nymvpn/util/FreepassCode.kt nym-vpn-android/app/src/test/java/net/nymtech/nymvpn/util/FreepassCodeTest.kt
git commit -m "feat(android): add validated free-pass code parser"
```

---

### Task 7: Android — Route, GeneratingMode, and Freepass generation flow

**Files:**
- Modify: `nym-vpn-android/app/src/main/java/net/nymtech/nymvpn/ui/Route.kt`
- Modify: `nym-vpn-android/app/src/main/java/net/nymtech/nymvpn/ui/screens/account/generating/GeneratingScreen.kt` (enum only)
- Modify: `nym-vpn-android/app/src/main/java/net/nymtech/nymvpn/ui/screens/account/generating/GeneratingViewModel.kt`

**Interfaces:**
- Consumes: `BackendManager.{isMnemonicStored, createAccount, applyFreepass}`; `parseFreepassCode` is **not** used here (the code arriving via the route is already validated at the scanner).
- Produces: `Route.FreepassScanner`; `Route.Generating(mode, code)`; `GeneratingMode.Freepass`; `GeneratingViewModel.freepassError: StateFlow<FreepassError?>` with `enum class FreepassError { INVALID, ALREADY_REDEEMED, GENERIC }`; `GeneratingViewModel.onFreepassErrorHandled()`.

- [ ] **Step 1: Extend the route + add scanner route**

In `Route.kt`, replace the `Generating` declaration and add the scanner route:

```kotlin
	@Serializable
	data class Generating(val mode: String = GeneratingMode.CreateAccount.name, val code: String? = null) : Route()

	@Serializable
	data object FreepassScanner : Route()
```

- [ ] **Step 2: Add the enum value**

In `GeneratingScreen.kt`, change the enum (last line):

```kotlin
enum class GeneratingMode { CreateAccount, DeepLinkLogin, Freepass }
```

- [ ] **Step 3: Add the Freepass branch to the view model**

In `GeneratingViewModel.kt`: read `code` from the route, add the error state, and add the `Freepass` init branch. Add fields near the existing flows:

```kotlin
	enum class FreepassError { INVALID, ALREADY_REDEEMED, GENERIC }

	private val _freepassError = MutableStateFlow<FreepassError?>(null)
	val freepassError = _freepassError.asStateFlow()

	fun onFreepassErrorHandled() { _freepassError.value = null }

	private val code: String? = savedStateHandle.toRoute<Route.Generating>().code
```

In `init`, add a branch (the `if (mode == CreateAccount) … else …` becomes a `when`):

```kotlin
		when (mode) {
			GeneratingMode.CreateAccount -> { /* existing CreateAccount block unchanged */ }
			GeneratingMode.Freepass -> startFreepassFlow()
			GeneratingMode.DeepLinkLogin -> Timber.tag(TAG).i("Generating started in DeepLinkLogin mode")
		}
```

Add the method:

```kotlin
	private fun startFreepassFlow() = viewModelScope.launch {
		val freepassCode = code
		if (freepassCode.isNullOrEmpty()) {
			Timber.tag(TAG).e("Freepass flow started without a code")
			_freepassError.value = FreepassError.GENERIC
			return@launch
		}
		runCatching {
			if (!backendManager.isMnemonicStored()) {
				backendManager.createAccount()
				Timber.tag(TAG).i("CreateAccountSuccess (freepass)")
			}
			backendManager.applyFreepass(freepassCode)
			Timber.tag(TAG).i("ApplyFreepassSuccess")
			val shouldShowTechnical = !settingsRepository.isTechnicalOptScreenCompleted()
			_pendingNavigation.value =
				if (shouldShowTechnical) Route.Main(authRoute = AuthRoute.TechOpt.routeName) else Route.Main()
		}.onFailure { t ->
			Timber.tag(TAG).w(t, "ApplyFreepassFailed")
			_freepassError.value = classifyFreepassError(t)
		}
	}

	private fun classifyFreepassError(t: Throwable): FreepassError {
		val msg = (t.message ?: "").lowercase()
		return when {
			msg.contains("already") || msg.contains("redeem") -> FreepassError.ALREADY_REDEEMED
			msg.contains("invalid") || msg.contains("not found") || msg.contains("notfound") -> FreepassError.INVALID
			else -> FreepassError.GENERIC
		}
	}
```

> `classifyFreepassError` is a **placeholder mapping** refined in Task 11 against the real API responses (`eJMWikx3EeU`, `hkB4sgMgfU8`). The uniffi error surfaces as an exception whose `message`/type carries the `VpnApiError` (`message`, `message_id`). In Task 11 you will log the actual thrown exception for both codes and replace the `when` with exact `message_id`/`message` matching.

- [ ] **Step 4: Compile**

Run: `cd nym-vpn-android && ./gradlew :app:compileGeneralDebugKotlin`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add nym-vpn-android/app/src/main/java/net/nymtech/nymvpn/ui/Route.kt nym-vpn-android/app/src/main/java/net/nymtech/nymvpn/ui/screens/account/generating
git commit -m "feat(android): add Freepass generation flow + scanner route"
```

---

### Task 8: Android — Generating screen Freepass error UI

Show an error dialog with "Try another code" / "Back to start" on freepass failure.

**Files:**
- Modify: `nym-vpn-android/app/src/main/java/net/nymtech/nymvpn/ui/screens/account/generating/GeneratingScreen.kt`
- Modify: `nym-vpn-android/app/src/main/res/values/strings.xml`

**Interfaces:**
- Consumes: `GeneratingViewModel.freepassError`, `onFreepassErrorHandled`; `Route.FreepassScanner`; `navigateAndForget`.

- [ ] **Step 1: Add strings**

In `app/src/main/res/values/strings.xml` (near `account_generating_error`):

```xml
	<string name="freepass_error_invalid_title">Invalid code</string>
	<string name="freepass_error_invalid_message">This free-pass code isn\'t valid. Check it and try again.</string>
	<string name="freepass_error_redeemed_title">Code already used</string>
	<string name="freepass_error_redeemed_message">This free-pass code has already been redeemed.</string>
	<string name="freepass_error_generic_title">Couldn\'t apply code</string>
	<string name="freepass_error_generic_message">Something went wrong applying this code. Please try again.</string>
	<string name="freepass_error_try_another">Try another code</string>
	<string name="freepass_error_back">Back to start</string>
	<string name="auth_scan_qr_button">Scan QR code</string>
	<string name="freepass_scanner_title">Scan your free-pass QR</string>
	<string name="freepass_scanner_instruction">Point your camera at the QR code, or enter the code manually below.</string>
	<string name="freepass_scanner_manual_label">Enter code manually</string>
	<string name="freepass_scanner_manual_hint">Free-pass code or link</string>
	<string name="freepass_scanner_submit">Continue</string>
	<string name="freepass_scanner_invalid_input">Enter a valid free-pass code</string>
	<string name="freepass_scanner_hint_no_match">That doesn\'t look like a Nym free-pass code — try the code field below.</string>
	<string name="freepass_scanner_camera_rationale">Camera access is needed to scan the QR code. You can also enter the code manually.</string>
	<string name="freepass_scanner_open_settings">Open settings</string>
```

- [ ] **Step 2: Render the error dialog in `GeneratingScreen`**

Add to the `GeneratingScreen` composable (after the existing `LaunchedEffect` blocks). Collect the error and show an `AlertDialog`:

```kotlin
	val freepassError by viewModel.freepassError.collectAsStateWithLifecycle()
	freepassError?.let { error ->
		val (titleRes, messageRes) = when (error) {
			GeneratingViewModel.FreepassError.INVALID ->
				R.string.freepass_error_invalid_title to R.string.freepass_error_invalid_message
			GeneratingViewModel.FreepassError.ALREADY_REDEEMED ->
				R.string.freepass_error_redeemed_title to R.string.freepass_error_redeemed_message
			GeneratingViewModel.FreepassError.GENERIC ->
				R.string.freepass_error_generic_title to R.string.freepass_error_generic_message
		}
		androidx.compose.material3.AlertDialog(
			onDismissRequest = { },
			title = { Text(stringResource(titleRes)) },
			text = { Text(stringResource(messageRes)) },
			confirmButton = {
				androidx.compose.material3.TextButton(onClick = {
					viewModel.onFreepassErrorHandled()
					navController.navigate(Route.FreepassScanner) {
						popUpTo(Route.Generating()) { inclusive = true }
					}
				}) { Text(stringResource(R.string.freepass_error_try_another)) }
			},
			dismissButton = {
				androidx.compose.material3.TextButton(onClick = {
					viewModel.onFreepassErrorHandled()
					navController.navigateAndForget(Route.Main(authRoute = AuthRoute.Welcome.routeName))
				}) { Text(stringResource(R.string.freepass_error_back)) }
			},
		)
	}
```

> Use `Text`, `stringResource`, `collectAsStateWithLifecycle` — already imported in this file. Add imports for `AlertDialog`/`TextButton` or use fully-qualified names as shown.

- [ ] **Step 3: Allow the animation to complete for Freepass**

In the existing `GeneratingContent` `onAnimationEnd` callback wiring inside `GeneratingScreen`, treat `Freepass` like `CreateAccount` so `pendingNavigation` fires:

```kotlin
		onAnimationEnd = {
			if (mode == GeneratingMode.CreateAccount || mode == GeneratingMode.Freepass) {
				animationEnded = true
			}
		},
```

And in `GeneratingContent`, the `isDeepLink` branch already drives the animation for non-deeplink modes, so `Freepass` reuses the create animation/steps unchanged.

- [ ] **Step 4: Compile**

Run: `cd nym-vpn-android && ./gradlew :app:compileGeneralDebugKotlin`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add nym-vpn-android/app/src/main/java/net/nymtech/nymvpn/ui/screens/account/generating/GeneratingScreen.kt nym-vpn-android/app/src/main/res/values/strings.xml
git commit -m "feat(android): freepass error dialog with retry/back"
```

---

### Task 9: Android — custom Compose scanner screen (camera + manual entry)

**Files:**
- Create: `nym-vpn-android/app/src/main/java/net/nymtech/nymvpn/ui/screens/account/scanner/FreepassScannerScreen.kt`
- Modify: `nym-vpn-android/app/build.gradle.kts` (add `accompanist-permissions`)
- Create: `nym-vpn-android/app/src/main/res/drawable/ic_qr_code.xml`

**Interfaces:**
- Consumes: `parseFreepassCode` (Task 6); `Route.Generating`, `Route.FreepassScanner`; `LocalNavController`; `zxing-android-embedded` `DecoratedBarcodeView`/`BarcodeView`, `DefaultDecoderFactory`, `BarcodeCallback`, `BarcodeResult`, `BarcodeFormat`.
- Produces: `@Composable fun FreepassScannerScreen()` registered in MainActivity (Task 10).

- [ ] **Step 1: Add the permissions dependency**

In `app/build.gradle.kts`, near the existing `implementation(libs.zxing.android.embedded)` (line ~252):

```kotlin
	implementation(libs.accompanist.permissions)
```

- [ ] **Step 2: Add the QR icon drawable**

Create `ic_qr_code.xml` (Material `qr_code_scanner` 24dp, tinted via `currentColor`/theme):

```xml
<vector xmlns:android="http://schemas.android.com/apk/res/android"
	android:width="24dp"
	android:height="24dp"
	android:viewportWidth="24"
	android:viewportHeight="24"
	android:tint="?attr/colorControlNormal">
	<path android:fillColor="@android:color/white"
		android:pathData="M9.5,6.5v3h-3v-3H9.5M11,5H5v6h6V5L11,5zM9.5,14.5v3h-3v-3H9.5M11,13H5v6h6V13L11,13zM17.5,6.5v3h-3v-3H17.5M19,5h-6v6h6V5L19,5zM13,13h1.5v1.5H13V13zM14.5,14.5H16V16h-1.5V14.5zM16,13h1.5v1.5H16V13zM13,16h1.5v1.5H13V16zM14.5,17.5H16V19h-1.5V17.5zM16,16h1.5v1.5H16V16zM17.5,14.5H19V16h-1.5V14.5zM17.5,17.5H19V19h-1.5V17.5zM22,7h-2V4h-3V2h5V7zM22,22v-5h-2v3h-3v2H22zM2,22h5v-2H4v-3H2V22zM2,2v5h2V4h3V2H2z" />
</vector>
```

- [ ] **Step 3: Implement the scanner screen**

Create `FreepassScannerScreen.kt`:

```kotlin
package net.nymtech.nymvpn.ui.screens.account.scanner

import android.Manifest
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.unit.dp
import androidx.compose.ui.viewinterop.AndroidView
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.LifecycleEventObserver
import androidx.lifecycle.compose.LocalLifecycleOwner
import com.google.accompanist.permissions.ExperimentalPermissionsApi
import com.google.accompanist.permissions.isGranted
import com.google.accompanist.permissions.rememberPermissionState
import com.google.accompanist.permissions.shouldShowRationale
import com.google.zxing.BarcodeFormat
import com.journeyapps.barcodescanner.BarcodeCallback
import com.journeyapps.barcodescanner.BarcodeResult
import com.journeyapps.barcodescanner.DecoratedBarcodeView
import com.journeyapps.barcodescanner.DefaultDecoderFactory
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.Route
import net.nymtech.nymvpn.ui.common.navigation.LocalNavController
import net.nymtech.nymvpn.ui.screens.account.generating.GeneratingMode
import net.nymtech.nymvpn.util.FreepassParseResult
import net.nymtech.nymvpn.util.parseFreepassCode

@OptIn(ExperimentalPermissionsApi::class)
@Composable
fun FreepassScannerScreen() {
	val navController = LocalNavController.current
	val cameraPermission = rememberPermissionState(Manifest.permission.CAMERA)
	var handled by rememberSaveable { mutableStateOf(false) }
	var manualInput by rememberSaveable { mutableStateOf("") }
	var manualError by remember { mutableStateOf(false) }

	fun proceed(code: String) {
		if (handled) return
		handled = true
		navController.navigate(Route.Generating(mode = GeneratingMode.Freepass.name, code = code)) {
			popUpTo(Route.FreepassScanner) { inclusive = true }
		}
	}

	fun onDecoded(raw: String) {
		when (val r = parseFreepassCode(raw)) {
			is FreepassParseResult.Valid -> proceed(r.code)
			FreepassParseResult.Invalid -> { /* ignore: keep scanning */ }
		}
	}

	Column(
		modifier = Modifier.fillMaxSize().padding(16.dp),
		horizontalAlignment = Alignment.CenterHorizontally,
	) {
		Text(stringResource(R.string.freepass_scanner_title), style = MaterialTheme.typography.titleMedium)
		Spacer(Modifier.height(8.dp))
		Text(stringResource(R.string.freepass_scanner_instruction), style = MaterialTheme.typography.bodyMedium)
		Spacer(Modifier.height(16.dp))

		if (cameraPermission.status.isGranted) {
			CameraScanner(onDecoded = ::onDecoded, modifier = Modifier.fillMaxWidth().weight(1f))
		} else {
			Column(Modifier.fillMaxWidth().weight(1f), horizontalAlignment = Alignment.CenterHorizontally) {
				Text(stringResource(R.string.freepass_scanner_camera_rationale))
				Spacer(Modifier.height(8.dp))
				if (cameraPermission.status.shouldShowRationale) {
					Button(onClick = { cameraPermission.launchPermissionRequest() }) { Text(stringResource(R.string.freepass_scanner_open_settings)) }
				} else {
					LaunchedEffect(Unit) { cameraPermission.launchPermissionRequest() }
				}
			}
		}

		Spacer(Modifier.height(16.dp))
		Text(stringResource(R.string.freepass_scanner_manual_label), style = MaterialTheme.typography.labelLarge)
		OutlinedTextField(
			value = manualInput,
			onValueChange = { manualInput = it; manualError = false },
			singleLine = true,
			isError = manualError,
			label = { Text(stringResource(R.string.freepass_scanner_manual_hint)) },
			keyboardOptions = KeyboardOptions(imeAction = ImeAction.Done),
			modifier = Modifier.fillMaxWidth(),
		)
		if (manualError) {
			Text(stringResource(R.string.freepass_scanner_invalid_input), color = MaterialTheme.colorScheme.error, style = MaterialTheme.typography.bodySmall)
		}
		Spacer(Modifier.height(8.dp))
		Button(
			onClick = {
				when (val r = parseFreepassCode(manualInput)) {
					is FreepassParseResult.Valid -> proceed(r.code)
					FreepassParseResult.Invalid -> manualError = true
				}
			},
			modifier = Modifier.fillMaxWidth(),
		) { Text(stringResource(R.string.freepass_scanner_submit)) }
	}
}

@Composable
private fun CameraScanner(onDecoded: (String) -> Unit, modifier: Modifier = Modifier) {
	val lifecycleOwner = LocalLifecycleOwner.current
	val currentOnDecoded by rememberUpdatedState(onDecoded)
	val barcodeView = remember { mutableStateOf<DecoratedBarcodeView?>(null) }

	AndroidView(
		modifier = modifier,
		factory = { context ->
			DecoratedBarcodeView(context).apply {
				barcodeView.decoderFactory = DefaultDecoderFactory(listOf(BarcodeFormat.QR_CODE))
				setStatusText("")
				decodeContinuous(object : BarcodeCallback {
					override fun barcodeResult(result: BarcodeResult) {
						result.text?.let { currentOnDecoded(it) }
					}
				})
				barcodeView.value = this
			}
		},
	)

	DisposableEffect(lifecycleOwner) {
		val observer = LifecycleEventObserver { _, event ->
			when (event) {
				Lifecycle.Event.ON_RESUME -> barcodeView.value?.resume()
				Lifecycle.Event.ON_PAUSE -> barcodeView.value?.pause()
				else -> Unit
			}
		}
		lifecycleOwner.lifecycle.addObserver(observer)
		onDispose {
			lifecycleOwner.lifecycle.removeObserver(observer)
			barcodeView.value?.pause()
		}
	}
}
```

> `DecoratedBarcodeView.barcodeView` is the inner `BarcodeView` exposing `decoderFactory`. If the 4.3.0 API differs, use `barcodeView` getter then `setDecoderFactory(...)`. The `setStatusText("")` removes the default footer.
> Remove the now-unused `com.journeyapps.barcodescanner.CaptureActivity` `<activity>` from the manifest only if nothing else references it — it is harmless to leave, so leave it.

- [ ] **Step 4: Compile**

Run: `cd nym-vpn-android && ./gradlew :app:compileGeneralDebugKotlin`
Expected: PASS.

- [ ] **Step 5: Commit**

```bash
git add nym-vpn-android/app/src/main/java/net/nymtech/nymvpn/ui/screens/account/scanner nym-vpn-android/app/build.gradle.kts nym-vpn-android/app/src/main/res/drawable/ic_qr_code.xml
git commit -m "feat(android): custom Compose free-pass scanner with manual entry"
```

---

### Task 10: Android — Welcome button + navigation wiring

**Files:**
- Modify: `nym-vpn-android/app/src/main/java/net/nymtech/nymvpn/ui/screens/auth/components/WelcomeView.kt`
- Modify: `nym-vpn-android/app/src/main/java/net/nymtech/nymvpn/ui/screens/auth/AuthComponent.kt`
- Modify: `nym-vpn-android/app/src/main/java/net/nymtech/nymvpn/ui/MainActivity.kt`

**Interfaces:**
- Consumes: `Route.FreepassScanner`; `FreepassScannerScreen`; `MainStyledButton`; `R.drawable.ic_qr_code`; `R.string.auth_scan_qr_button`.

- [ ] **Step 1: Add the button to `WelcomeView`**

Add an `onScanQrClick: () -> Unit` param to `WelcomeView`, and a third button after the Login button (before `PrivacyText()`), with a leading icon:

```kotlin
fun WelcomeView(onLoginClick: () -> Unit, onSignUpClick: () -> Unit, onScanQrClick: () -> Unit, modifier: Modifier = Modifier) {
```

```kotlin
		MainStyledButton(
			onClick = onScanQrClick,
			content = {
				Row(
					horizontalArrangement = Arrangement.spacedBy(8.dp),
					verticalAlignment = Alignment.CenterVertically,
				) {
					Icon(
						imageVector = ImageVector.vectorResource(R.drawable.ic_qr_code),
						contentDescription = null,
					)
					Text(
						stringResource(R.string.auth_scan_qr_button),
						style = MaterialTheme.typography.titleMedium,
					)
				}
			},
			modifier = Modifier.fillMaxWidth().height(48.dp.scaledHeight()),
			shape = RoundedCornerShape(12.dp),
		)
```

Add imports: `androidx.compose.foundation.layout.Row`, `androidx.compose.ui.Alignment` (present). Update the `@Preview` `WelcomeView(...)` call to pass `onScanQrClick = {}`.

- [ ] **Step 2: Wire the button in `AuthComponent`**

In the `composable<AuthRoute.Welcome>` block, pass the handler — mirror the anonymous flow's `onAuthSuccess()` + root navigation:

```kotlin
			WelcomeView(
				onLoginClick = { localNavController.navigate(AuthRoute.Login) },
				onSignUpClick = { localNavController.navigate(AuthRoute.SignUp) },
				onScanQrClick = {
					onAuthSuccess()
					rootNavController.navigate(Route.FreepassScanner)
				},
			)
```

- [ ] **Step 3: Register the scanner route in `MainActivity`**

Add next to `composable<Route.Generating>` (~line 300):

```kotlin
								composable<Route.FreepassScanner> {
									net.nymtech.nymvpn.ui.screens.account.scanner.FreepassScannerScreen()
								}
```

(or add the import and use `FreepassScannerScreen()`.)

- [ ] **Step 4: Build the debug APK**

Run: `cd nym-vpn-android && ./gradlew :app:assembleGeneralDebug`
Expected: BUILD SUCCESSFUL.

- [ ] **Step 5: Commit**

```bash
git add nym-vpn-android/app/src/main/java/net/nymtech/nymvpn/ui/screens/auth nym-vpn-android/app/src/main/java/net/nymtech/nymvpn/ui/MainActivity.kt
git commit -m "feat(android): add Scan QR code button to welcome screen"
```

---

### Task 11: Error mapping verification + end-to-end manual test

Lock down the invalid-vs-redeemed distinction against the real API and verify the whole flow.

**Files:**
- Modify (if needed): `nym-vpn-android/app/src/main/java/net/nymtech/nymvpn/ui/screens/account/generating/GeneratingViewModel.kt` (`classifyFreepassError`)

- [ ] **Step 1: Capture the real errors**

On a device/emulator, temporarily add `Timber.tag(TAG).e(t, "freepass-raw-error: ${'$'}{t::class.java} ${'$'}{t.message}")` inside `startFreepassFlow`'s `onFailure`. Run the flow with the already-redeemed code `hkB4sgMgfU8` and capture the thrown exception type + message via `adb logcat`. (The uniffi error is an `AccountCommandError`-derived exception carrying the API `message`/`message_id`.)

- [ ] **Step 2: Refine `classifyFreepassError`**

Replace the heuristic `when` with exact matching on the observed exception class / `message_id` / `message` for `ALREADY_REDEEMED` and `INVALID`. If the two cases are indistinguishable from the response, collapse the dialog to `GENERIC` only (and drop the unused strings). Remove the temporary log line.

- [ ] **Step 3: Manual end-to-end checklist**

Run on a device/emulator (`./gradlew :app:installGeneralDebug`):
- [ ] Welcome shows three buttons; "Scan QR code" has the QR icon.
- [ ] Tap it → scanner screen opens; camera permission prompt appears; deny → rationale + manual field still usable.
- [ ] Scan a QR encoding `https://nym.com/account/freepass?code=eJMWikx3EeU` → auto-continues (no tap) → Generating → lands on TechOpt/Main; user can connect.
- [ ] Point camera at an unrelated QR → ignored, no navigation.
- [ ] Re-run; type `hkB4sgMgfU8` manually → "Code already used"; "Try another code" returns to scanner without creating a second account (verify only one account via logs / no duplicate create); "Back to start" returns to Welcome.
- [ ] Type garbage (`abc';DROP`) manually → inline "Enter a valid free-pass code"; nothing sent.

- [ ] **Step 4: Commit**

```bash
git add nym-vpn-android/app/src/main/java/net/nymtech/nymvpn/ui/screens/account/generating/GeneratingViewModel.kt
git commit -m "fix(android): map freepass API errors to user-facing states"
```

---

## Self-Review

**Spec coverage:**
- Third "Scan QR code" button + icon → Task 10 / Task 9 (drawable). ✓
- Camera opens + QR scanner starts → Task 9. ✓
- F-Droid-accepted library (zxing) → already present; Task 9 uses it. ✓
- Reads `https://nym.com/...?code=…` and bare code, varying size → Task 6 parser. ✓
- Create + register account (existing) → reused in Task 7. ✓
- Apply freepass (Rust→uniffi→Android) → Tasks 1–5. ✓
- App authorized to call endpoint → JWT minted in Rust client (existing); no new network permission needed (app already has internet + the API client). ✓
- Success → next screen / connect → Task 7 navigation. ✓
- Failure (invalid / already-used) → retry or back → Task 8. ✓
- Manual entry → Task 9. ✓
- Auto-continue on valid QR → Task 9 `onDecoded`/`proceed`. ✓
- Anti-injection (allow-list) → Task 6. ✓

**Placeholders:** `classifyFreepassError` is explicitly a heuristic resolved in Task 11 with real data — flagged, not silent. Parser URL-detection has a fallback note. No other TODOs.

**Type consistency:** `parseFreepassCode`/`FreepassParseResult.{Valid,Invalid}` consistent across Tasks 6, 9. `GeneratingMode.Freepass`, `Route.Generating(mode, code)`, `Route.FreepassScanner`, `FreepassError` consistent across Tasks 7, 8, 9, 10. `applyFreepass(code)` consistent across Rust (Tasks 1–3), binding (Task 4), Android (Task 5).
