package net.nymtech.nymvpn.manager.pairing

import androidx.biometric.BiometricPrompt
import kotlinx.coroutines.CancellationException
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.TimeoutCancellationException
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.catch
import kotlinx.coroutines.flow.filterIsInstance
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.launch
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import kotlinx.coroutines.withTimeout
import net.nymtech.nymvpn.di.qualifiers.ApplicationScope
import net.nymtech.nymvpn.manager.backend.BackendManager
import net.nymtech.nymvpn.manager.backend.VpnServiceConnectionManager
import net.nymtech.nymvpn.manager.deviceauth.DeviceAuthGate
import net.nymtech.nymvpn.manager.pairing.model.PairingState
import net.nymtech.nymvpn.manager.pairing.model.activeSessionId
import net.nymtech.vpn.backend.api.VpnServiceApi
import net.nymtech.vpn.model.VpnServiceEvent
import net.nymtech.vpn.model.pairing.PairingException
import net.nymtech.vpn.model.pairing.PairingFailure
import nym_vpn_lib_types.PairingResult
import timber.log.Timber
import javax.inject.Inject
import javax.inject.Singleton
import kotlin.time.Duration.Companion.minutes
import kotlin.time.Duration.Companion.seconds
import kotlin.time.TimeSource

@Singleton
class NymPairingManager @Inject constructor(
	private val connectionManager: VpnServiceConnectionManager,
	private val backendManager: BackendManager,
	private val deviceAuthGate: DeviceAuthGate,
	@ApplicationScope appScope: CoroutineScope,
) : PairingManager {

	private val _state = MutableStateFlow<PairingState>(PairingState.Idle)
	override val state: StateFlow<PairingState> = _state.asStateFlow()

	private val mutex = Mutex()

	// The core doesn't replay pairing events, so they are collected for the app's lifetime.
	init {
		appScope.launch { collectEvents() }
	}

	override suspend fun startShowingCode() {
		mutex.withLock {
			cancelActiveSession()
			_state.value = PairingState.StartingCode
			_state.value = try {
				val qr = call { it.startPairing() }
				PairingState.ShowingCode(qr.url, qr.sessionId, TimeSource.Monotonic.markNow() + CODE_LIFETIME)
			} catch (e: PairingException) {
				PairingState.Failed(e.failure)
			}
		}
	}

	override suspend fun confirmCode(cryptoObject: BiometricPrompt.CryptoObject?): Boolean = mutex.withLock {
		val confirm = _state.value as? PairingState.ConfirmCode ?: return@withLock true
		if (!deviceAuthGate.verify(cryptoObject)) return@withLock false
		_state.value = PairingState.SharingAccount
		_state.value = try {
			call { it.verifiedPairingPin(confirm.sessionId) }
			Timber.tag(TAG).i("Account shared, session %s", confirm.sessionId)
			PairingState.AccountShared
		} catch (e: PairingException) {
			PairingState.Failed(e.failure)
		}
		true
	}

	override suspend fun onScannedKey(publicKey: String): PairingFailure? = mutex.withLock {
		cancelActiveSession()
		try {
			val confirmation = call { it.scannedPairingKey(publicKey) }
			_state.value = PairingState.WaitingForApproval(confirmation.pinCode, confirmation.sessionId)
			null
		} catch (e: PairingException) {
			// The scanner shows the failure inline.
			_state.value = PairingState.Idle
			e.failure
		}
	}

	override suspend fun cancel() {
		mutex.withLock {
			cancelActiveSession()
			_state.value = PairingState.Idle
		}
	}

	private suspend fun cancelActiveSession() {
		if (_state.value.activeSessionId == null) return
		_state.value = PairingState.Idle
		try {
			call { it.cancelPairing() }
		} catch (e: PairingException) {
			Timber.tag(TAG).w("Cancel failed: %s", e.failure)
		}
	}

	@OptIn(ExperimentalCoroutinesApi::class)
	private suspend fun collectEvents() {
		connectionManager.apiFlow
			.filterNotNull()
			.flatMapLatest { it.events }
			.filterIsInstance<VpnServiceEvent.Pairing>()
			.catch { Timber.tag(TAG).e(it, "Pairing events stream failed") }
			.collect { event -> mutex.withLock { onEvent(event) } }
	}

	private suspend fun onEvent(event: VpnServiceEvent.Pairing) {
		val current = _state.value
		// Ignores ended sessions, including the Cancelled that follows our own cancel.
		if (current.activeSessionId != event.sessionId) return
		Timber.tag(TAG).i("Event %s in %s", event, current)
		when (val result = event.result) {
			is PairingResult.PinCode -> if (current is PairingState.ShowingCode) {
				_state.value = PairingState.ConfirmCode(result.v1, current.sessionId)
			}
			PairingResult.StoredMnemonic -> if (current is PairingState.WaitingForApproval) {
				runCatching { backendManager.refreshStoredAccount() }
					.onFailure { Timber.tag(TAG).w("Refresh after pairing failed: %s", it::class.java.simpleName) }
				_state.value = PairingState.SignedIn
			}
			PairingResult.TimedOut -> _state.value = PairingState.Expired
			PairingResult.Cancelled -> _state.value = PairingState.Failed(PairingFailure.SESSION_ENDED)
			is PairingResult.Error -> _state.value = PairingState.Failed(PairingFailure.UNKNOWN)
		}
	}

	private suspend fun <T> call(block: suspend (VpnServiceApi) -> T): T = try {
		withTimeout(CALL_TIMEOUT) { connectionManager.withApi(block = block) }
	} catch (e: PairingException) {
		throw e
	} catch (e: TimeoutCancellationException) {
		throw PairingException(PairingFailure.SERVER, e)
	} catch (e: CancellationException) {
		throw e
	} catch (e: Exception) {
		Timber.tag(TAG).w("Pairing call failed: %s", e::class.java.simpleName)
		throw PairingException(PairingFailure.UNKNOWN, e)
	}

	private companion object {
		const val TAG = "pairing"
		val CODE_LIFETIME = 5.minutes
		val CALL_TIMEOUT = 30.seconds
	}
}
