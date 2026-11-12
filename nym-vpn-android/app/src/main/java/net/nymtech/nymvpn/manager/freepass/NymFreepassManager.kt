package net.nymtech.nymvpn.manager.freepass

import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.NonCancellable
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import kotlinx.coroutines.withContext
import kotlinx.coroutines.withTimeoutOrNull
import net.nymtech.nymvpn.manager.backend.BackendManager
import net.nymtech.nymvpn.manager.backend.VpnServiceConnectionManager
import net.nymtech.vpn.model.freepass.FreepassError
import net.nymtech.vpn.model.freepass.FreepassResult
import timber.log.Timber
import javax.inject.Inject
import javax.inject.Singleton
import kotlin.time.Duration.Companion.seconds

@Singleton
class NymFreepassManager @Inject constructor(private val connectionManager: VpnServiceConnectionManager, private val backendManager: BackendManager) : FreepassManager {

	private val mutex = Mutex()

	// Not cancellable, so a local account never outlives a failed redeem.
	override suspend fun redeem(code: String): FreepassResult = withContext(NonCancellable) {
		mutex.withLock {
			if (backendManager.isMnemonicStored()) applyToStoredAccount(code) else createAccount(code)
		}
	}

	// A local account is registered on the API only on first connect, so redeem registers it with the pass.
	private suspend fun applyToStoredAccount(code: String): FreepassResult {
		val result = connectionManager.withApiResult(FreepassError, FreepassResult::Failed) { it.applyFreepass(code) }
		if (result != FreepassResult.Failed(FreepassError.ACCOUNT_NOT_REGISTERED)) return result
		return connectionManager.withApiResult(FreepassError, FreepassResult::Failed) { it.redeemFreepass(code) }
	}

	private suspend fun createAccount(code: String): FreepassResult {
		val created = try {
			// The core waits for its initialization, so the wait is bounded.
			withTimeoutOrNull(ACCOUNT_CREATION_TIMEOUT) { backendManager.createAccount() } != null
		} catch (e: CancellationException) {
			throw e
		} catch (e: Exception) {
			Timber.tag(TAG).w("Account creation failed: %s", e::class.java.simpleName)
			false
		}
		if (!created) return FreepassResult.Failed(FreepassError.UNKNOWN)
		val result = connectionManager.withApiResult(FreepassError, FreepassResult::Failed) { it.redeemFreepass(code) }
		// A timeout or unknown failure may still have redeemed, so only a confirmed rejection drops the local account.
		if (result is FreepassResult.Failed && result.error.isRejection) {
			runCatching { backendManager.removeMnemonic() }
				.onFailure { Timber.tag(TAG).w("Removing the local account failed: %s", it::class.java.simpleName) }
		}
		return result
	}

	private val FreepassError.isRejection: Boolean
		get() = this != FreepassError.SERVER && this != FreepassError.UNKNOWN

	private companion object {
		const val TAG = "freepass"
		val ACCOUNT_CREATION_TIMEOUT = 30.seconds
	}
}
