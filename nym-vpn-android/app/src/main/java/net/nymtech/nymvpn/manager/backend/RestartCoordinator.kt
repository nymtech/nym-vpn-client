package net.nymtech.nymvpn.manager.backend

import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.channels.BufferOverflow
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableSharedFlow
import kotlinx.coroutines.flow.asSharedFlow
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.flow.debounce
import kotlinx.coroutines.flow.dropWhile
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.launch
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import kotlinx.coroutines.withTimeout
import net.nymtech.vpn.backend.Tunnel

/**
 * Debounced restart coordinator for the tunnel.
 */
class RestartCoordinator(
	private val scope: CoroutineScope,
	private val dispatcher: CoroutineDispatcher,
	private val stateFlow: Flow<*>,
	private val getState: () -> Tunnel.State,
	private val stopTunnel: suspend () -> Unit,
	private val startTunnel: suspend () -> Unit,
	private val onRestartStarted: (Boolean) -> Unit,
) {
	private val restartMutex = Mutex()

	private data class RestartRequest(val shouldResetConnectionTime: Boolean)

	private val restartRequests = MutableSharedFlow<RestartRequest>(
		extraBufferCapacity = 1,
		onBufferOverflow = BufferOverflow.DROP_OLDEST,
	)

	private val _restartStartedEvents = MutableSharedFlow<Unit>(
		extraBufferCapacity = 1,
		onBufferOverflow = BufferOverflow.DROP_OLDEST,
	)
	val restartStartedEvents: Flow<Unit> = _restartStartedEvents.asSharedFlow()

	fun start() {
		scope.launch(dispatcher) {
			restartRequests
				.debounce(500)
				.collectLatest { restartNow(it.shouldResetConnectionTime) }
		}
	}

	fun requestRestartDebounced(shouldResetConnectionTime: Boolean) {
		restartRequests.tryEmit(RestartRequest(shouldResetConnectionTime))
	}

	suspend fun restartNow(shouldResetConnectionTime: Boolean) = restartMutex.withLock {
		onRestartStarted(shouldResetConnectionTime)
		_restartStartedEvents.tryEmit(Unit)

		val currentState = getState()
		if (currentState != Tunnel.State.Down) {
			stopTunnel()
			withTimeout(15_000) {
				stateFlow.dropWhile { true }.first()
			}
		}

		delay(2_500)
		startTunnel()
	}
}
