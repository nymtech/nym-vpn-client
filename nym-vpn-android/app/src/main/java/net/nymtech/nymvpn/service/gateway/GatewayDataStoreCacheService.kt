package net.nymtech.nymvpn.service.gateway

import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.CoroutineStart
import kotlinx.coroutines.Deferred
import kotlinx.coroutines.async
import kotlinx.coroutines.currentCoroutineContext
import kotlinx.coroutines.delay
import kotlinx.coroutines.ensureActive
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.withTimeout
import net.nymtech.nymvpn.data.GatewayRepository
import net.nymtech.nymvpn.manager.backend.BackendManager
import nym_vpn_lib_types.GatewayType
import timber.log.Timber
import java.util.concurrent.ConcurrentHashMap
import kotlin.time.Duration.Companion.minutes
import kotlin.time.Duration.Companion.seconds
import kotlin.time.TimeMark
import kotlin.time.TimeSource

// Refreshes the gateway cache from the backend with dedup, freshness check, retries and syncing state.
class GatewayDataStoreCacheService(private val gatewayRepository: GatewayRepository, private val backend: BackendManager, private val appScope: CoroutineScope) : GatewayCacheService {

	companion object {
		private const val TAG = "gateway-cache"
		private const val MAX_ATTEMPTS = 3
		private val FRESH_FOR = 5.minutes
		private val FETCH_TIMEOUT = 30.seconds
		private val RETRY_DELAY = 2.seconds
	}

	private val _syncingTypes = MutableStateFlow<Set<GatewayType>>(emptySet())
	override val syncingTypes: StateFlow<Set<GatewayType>> = _syncingTypes.asStateFlow()

	private val inFlight = mutableMapOf<GatewayType, Deferred<Result<Unit>>>()

	// In-memory on purpose: every app start fetches a fresh list.
	private val lastUpdated = ConcurrentHashMap<GatewayType, TimeMark>()

	override suspend fun refresh(type: GatewayType, force: Boolean): Result<Unit> {
		if (!force && isFresh(type)) return Result.success(Unit)
		// Callers join an ongoing request; it runs in app scope so it outlives the screen that started it.
		val request = synchronized(inFlight) {
			inFlight.getOrPut(type) { appScope.async(start = CoroutineStart.LAZY) { fetch(type) } }
		}
		return request.await()
	}

	override suspend fun clear() {
		lastUpdated.clear()
		gatewayRepository.clear()
	}

	private fun isFresh(type: GatewayType): Boolean = lastUpdated[type]?.let { it.elapsedNow() < FRESH_FOR } == true

	private suspend fun fetch(type: GatewayType): Result<Unit> {
		_syncingTypes.update { it + type }
		try {
			var result: Result<Unit> = Result.failure(IllegalStateException("Gateway fetch not attempted"))
			for (attempt in 1..MAX_ATTEMPTS) {
				if (attempt > 1) delay(RETRY_DELAY * (attempt - 1))
				result = runCatching {
					val gateways = withTimeout(FETCH_TIMEOUT) { backend.getGateways(type) }
					check(gateways.isNotEmpty()) { "Empty gateway list" }
					gatewayRepository.setGateways(type, gateways)
				}
				currentCoroutineContext().ensureActive()
				if (result.isSuccess) {
					lastUpdated[type] = TimeSource.Monotonic.markNow()
					Timber.tag(TAG).i("GatewayCacheUpdated type=%s attempt=%d", type, attempt)
					return result
				}
				Timber.tag(TAG).w(result.exceptionOrNull(), "GatewayFetchFailed type=%s attempt=%d", type, attempt)
			}
			return result
		} finally {
			synchronized(inFlight) { inFlight.remove(type) }
			_syncingTypes.update { it - type }
		}
	}
}
