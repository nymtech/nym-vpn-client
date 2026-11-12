package net.nymtech.nymvpn.manager.freepass

import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.NonCancellable
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import kotlinx.coroutines.withContext
import net.nymtech.nymvpn.manager.backend.BackendManager
import net.nymtech.nymvpn.manager.backend.VpnServiceConnectionManager
import net.nymtech.vpn.model.freepass.FreepassError
import net.nymtech.vpn.model.freepass.FreepassResult
import timber.log.Timber
import javax.inject.Inject
import javax.inject.Singleton

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
		try {
			backendManager.createAccount()
		} catch (e: CancellationException) {
			throw e
		} catch (e: Exception) {
			Timber.tag(TAG).w("Account creation failed: %s", e::class.java.simpleName)
			return FreepassResult.Failed(FreepassError.UNKNOWN)
		}
		val result = connectionManager.withApiResult(FreepassError, FreepassResult::Failed) { it.redeemFreepass(code) }
		if (result is FreepassResult.Failed) {
			// Nothing was created on the API, so drop the local account.
			runCatching { backendManager.removeMnemonic() }
				.onFailure { Timber.tag(TAG).w("Removing the local account failed: %s", it::class.java.simpleName) }
		}
		return result
	}

	private companion object {
		const val TAG = "freepass"
	}
}
