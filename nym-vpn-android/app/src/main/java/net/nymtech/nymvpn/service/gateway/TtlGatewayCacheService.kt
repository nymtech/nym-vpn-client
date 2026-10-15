package net.nymtech.nymvpn.service.gateway

import kotlinx.coroutines.async
import kotlinx.coroutines.awaitAll
import kotlinx.coroutines.coroutineScope
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import net.nymtech.nymvpn.data.GatewayRepository
import net.nymtech.vpn.backend.Tunnel
import net.nymtech.vpn.model.NymGateway
import nym_vpn_lib_types.GatewayType
import timber.log.Timber

class TtlGatewayCacheService(
	private val repository: GatewayRepository,
	private val fetch: suspend (GatewayType) -> List<NymGateway>,
	private val ttlMs: Long = DEFAULT_TTL_MS,
	private val nowMs: () -> Long = System::currentTimeMillis,
) : GatewayCacheService {

	companion object {
		private const val TAG = "gateway-cache"
		const val DEFAULT_TTL_MS = 10 * 60 * 1_000L
	}

	private val lastFetched = HashMap<GatewayType, Long>()
	private val locks = GatewayType.entries.associateWith { Mutex() }

	override suspend fun refresh(type: GatewayType, force: Boolean): Result<Unit> = locks.getValue(type).withLock {
		val last = lastFetched[type]
		if (!force && last != null && nowMs() - last < ttlMs) return Result.success(Unit)
		runCatching {
			val gateways = fetch(type)
			when (type) {
				GatewayType.MIXNET_ENTRY -> repository.setEntryGateways(gateways)
				GatewayType.MIXNET_EXIT -> repository.setExitGateways(gateways)
				GatewayType.WG -> repository.setWgGateways(gateways)
			}
			lastFetched[type] = nowMs()
		}.onFailure { Timber.tag(TAG).e(it, "GatewayRefreshFailed type=%s", type) }
	}

	override suspend fun refreshForMode(mode: Tunnel.Mode, force: Boolean) {
		val types = when (mode) {
			Tunnel.Mode.TWO_HOP_MIXNET -> listOf(GatewayType.WG)
			Tunnel.Mode.FIVE_HOP_MIXNET -> listOf(GatewayType.MIXNET_ENTRY, GatewayType.MIXNET_EXIT)
		}
		coroutineScope { types.map { async { refresh(it, force) } }.awaitAll() }
	}
}
