package net.nymtech.nymvpn.manager.freepass

import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import net.nymtech.nymvpn.manager.backend.BackendManager
import net.nymtech.vpn.model.account.CreateAccountError
import net.nymtech.vpn.model.account.CreateAccountResult
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
			true -> backendManager.redeemFreepass(code)
			false -> when (val result = backendManager.createAccount(freepass = code)) {
				CreateAccountResult.Created -> FreepassResult.Redeemed
				is CreateAccountResult.Failed -> FreepassResult.Failed(result.error.toFreepassError())
			}
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

	private fun CreateAccountError.toFreepassError(): FreepassError = when (this) {
		CreateAccountError.FREEPASS_INVALID -> FreepassError.INVALID_CODE
		CreateAccountError.FREEPASS_ALREADY_REDEEMED -> FreepassError.ALREADY_REDEEMED
		CreateAccountError.SUBSCRIPTION_ACTIVE -> FreepassError.SUBSCRIPTION_ACTIVE
		CreateAccountError.RATE_LIMITED -> FreepassError.RATE_LIMITED
		CreateAccountError.OFFLINE -> FreepassError.OFFLINE
		CreateAccountError.SERVER -> FreepassError.SERVER
		CreateAccountError.UNKNOWN -> FreepassError.UNKNOWN
	}

	private companion object {
		const val TAG = "freepass"
	}
}
