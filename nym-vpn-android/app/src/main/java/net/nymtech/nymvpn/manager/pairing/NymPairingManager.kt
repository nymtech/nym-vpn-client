package net.nymtech.nymvpn.manager.pairing

import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.NonCancellable
import kotlinx.coroutines.currentCoroutineContext
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.catch
import kotlinx.coroutines.flow.filterIsInstance
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.isActive
import kotlinx.coroutines.launch
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import kotlinx.coroutines.withContext
import net.nymtech.nymvpn.di.qualifiers.ApplicationScope
import net.nymtech.nymvpn.manager.backend.BackendManager
import net.nymtech.nymvpn.manager.backend.VpnServiceConnectionManager
import net.nymtech.nymvpn.manager.pairing.model.PairingState
import net.nymtech.nymvpn.manager.pairing.model.activeSessionId
import net.nymtech.vpn.backend.api.VpnServiceApi
import net.nymtech.vpn.model.VpnServiceEvent
import net.nymtech.vpn.model.pairing.PairingCommandResult
import net.nymtech.vpn.model.pairing.PairingError
import nym_vpn_lib_types.PairingResult
import timber.log.Timber
import javax.inject.Inject
import javax.inject.Singleton
import kotlin.time.Duration.Companion.minutes
import kotlin.time.TimeSource

@Singleton
class NymPairingManager @Inject constructor(private val connectionManager: VpnServiceConnectionManager, private val backendManager: BackendManager, @ApplicationScope appScope: CoroutineScope) :
	PairingManager {

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
			_state.value = when (val result = call { it.startPairing() }) {
				is PairingCommandResult.Ok -> PairingState.ShowingCode(result.value.url, result.value.sessionId, TimeSource.Monotonic.markNow() + CODE_LIFETIME)
				is PairingCommandResult.Failed -> PairingState.Failed(result.error)
			}
		}
	}

	override suspend fun confirmCode() = mutex.withLock {
		val confirm = _state.value as? PairingState.ConfirmCode ?: return@withLock
		_state.value = PairingState.SharingAccount
		_state.value = when (val result = withContext(NonCancellable) { call { it.verifiedPairingPin(confirm.sessionId) } }) {
			is PairingCommandResult.Ok -> {
				Timber.tag(TAG).i("Account shared, session %s", confirm.sessionId)
				PairingState.AccountShared
			}
			is PairingCommandResult.Failed -> PairingState.Failed(result.error)
		}
	}

	override suspend fun onScannedKey(publicKey: String): PairingCommandResult<Unit> = mutex.withLock {
		cancelActiveSession()
		// The core keeps the session even if the scanner closes meanwhile, so it's always recorded.
		when (val result = withContext(NonCancellable) { call { it.scannedPairingKey(publicKey) } }) {
			is PairingCommandResult.Ok -> {
				_state.value = PairingState.WaitingForApproval(result.value.pinCode, result.value.sessionId)
				if (!currentCoroutineContext().isActive) withContext(NonCancellable) { cancelActiveSession() }
				PairingCommandResult.Ok(Unit)
			}
			is PairingCommandResult.Failed -> {
				// The scanner shows the error inline.
				_state.value = PairingState.Idle
				result
			}
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
		val result = call { it.cancelPairing() }
		if (result is PairingCommandResult.Failed) Timber.tag(TAG).w("Cancel failed: %s", result.error)
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
			PairingResult.Cancelled -> _state.value = PairingState.Failed(PairingError.SESSION_ENDED)
			is PairingResult.Error -> _state.value = PairingState.Failed(PairingError.UNKNOWN)
		}
	}

	private suspend fun <T> call(block: suspend (VpnServiceApi) -> PairingCommandResult<T>): PairingCommandResult<T> =
		connectionManager.withApiResult(PairingError, { PairingCommandResult.Failed(it) }, block)

	private companion object {
		const val TAG = "pairing"
		val CODE_LIFETIME = 5.minutes
	}
}
