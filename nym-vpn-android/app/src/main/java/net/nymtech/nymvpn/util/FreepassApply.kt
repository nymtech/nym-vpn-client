package net.nymtech.nymvpn.util

import net.nymtech.nymvpn.manager.backend.BackendManager
import timber.log.Timber

private const val TAG = "freepass-apply"

/** Categorised free-pass apply failure, used to pick the user-facing message. */
enum class FreepassError { INVALID, ALREADY_REDEEMED, GENERIC }

/**
 * Ensure the stored account is registered with the API, then apply the free-pass [code].
 *
 * This never creates an account — the caller is responsible for that. It registers the
 * already-stored account if needed (the free-pass endpoint returns "Account not found"
 * otherwise) and then applies the code. Throws if applying fails.
 */
suspend fun BackendManager.ensureRegisteredAndApplyFreepass(code: String) {
	registerForFreepass()
	applyFreepass(code)
}

/**
 * Register the stored account with the API.
 *
 * `register_android_account` is not idempotent under client-side retries / a concurrent
 * controller sync: the first POST creates the account server-side, a retry then returns
 * "account already exists". Either way the account now exists — all we need before applying a
 * free-pass — so an "already exists" response is treated as success. No purchase token is used.
 */
suspend fun BackendManager.registerForFreepass() {
	try {
		registerAccount(null)
		Timber.tag(TAG).i("RegisterAccountSuccess (freepass)")
	} catch (t: Throwable) {
		if (t.freepassMessageContains("already exists", "already-exists")) {
			Timber.tag(TAG).i("Account already registered (freepass) — continuing")
		} else {
			throw t
		}
	}
}

/** Map an apply failure to a user-facing category. */
fun classifyFreepassError(t: Throwable): FreepassError = when {
	t.freepassMessageContains("already", "redeem") -> FreepassError.ALREADY_REDEEMED
	t.freepassMessageContains("invalid", "not found", "notfound") -> FreepassError.INVALID
	else -> FreepassError.GENERIC
}

/**
 * The uniffi error is a handle-backed object whose Kotlin `message`/`cause` are null; the real
 * detail (Rust Display via display_chain(), incl. the API message and message_id such as
 * "...account-already-exists") is only available from toString(). Match against toString() and
 * the cause chain so both uniffi errors and ordinary exceptions are covered.
 */
private fun Throwable.freepassMessageContains(vararg needles: String): Boolean {
	val sb = StringBuilder()
	var cur: Throwable? = this
	while (cur != null) {
		sb.append(cur.toString()).append(' ')
		cur.message?.let { sb.append(it).append(' ') }
		cur = cur.cause
	}
	val all = sb.toString().lowercase()
	return needles.any { all.contains(it) }
}
