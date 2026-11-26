package net.nymtech.nymvpn.manager.freepass

import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import net.nymtech.nymvpn.manager.backend.BackendManager
import net.nymtech.vpn.model.freepass.FreepassError
import net.nymtech.vpn.model.freepass.FreepassResult
import timber.log.Timber
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class NymFreepassManager @Inject constructor(private val backendManager: BackendManager) : FreepassManager {

	private val mutex = Mutex()

	override suspend fun redeem(code: String): FreepassResult = mutex.withLock {
		when (isAccountStored()) {
			true -> backendManager.redeemFreepassForStoredAccount(code)
			false -> backendManager.redeemFreepassForNewAccount(code)
			null -> FreepassResult.Failed(FreepassError.UNKNOWN)
		}
	}

	private suspend fun isAccountStored(): Boolean? = try {
		backendManager.isMnemonicStored()
	} catch (e: CancellationException) {
		throw e
	} catch (e: Exception) {
		Timber.tag(TAG).w("Account lookup failed: %s", e::class.java.simpleName)
		null
	}

	private companion object {
		const val TAG = "freepass"
	}
}
