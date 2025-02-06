package net.nymtech.nymvpn.service.gateway

import net.nymtech.nymvpn.data.GatewayRepository
import net.nymtech.nymvpn.manager.backend.BackendManager
import nym_vpn_lib.GatewayType
import timber.log.Timber
import javax.inject.Inject

class GatewayDataStoreCacheService @Inject constructor(
	private val gatewayRepository: GatewayRepository,
	private val backend: BackendManager,
) : GatewayCacheService {
	override suspend fun updateExitGatewayCache(): Result<Unit> {
		return runCatching {
			val countries = backend.getGateways(GatewayType.MIXNET_EXIT)
			gatewayRepository.setExitGateways(countries)
			Timber.d("Updated mixnet exit countries cache")
		}.onFailure {
			Timber.e(it)
		}
	}

	override suspend fun updateEntryGatewayCache(): Result<Unit> {
		return runCatching {
			val countries = backend.getGateways(GatewayType.MIXNET_ENTRY)
			gatewayRepository.setEntryGateways(countries)
			Timber.d("Updated mixnet entry countries cache")
		}.onFailure {
			Timber.e(it)
		}
	}

	override suspend fun updateWgGatewayCache(): Result<Unit> {
		return kotlin.runCatching {
			val countries = backend.getGateways(GatewayType.WG)
			gatewayRepository.setWgGateways(countries)
			Timber.d("Updated wg countries cache")
		}.onFailure {
			Timber.e(it)
		}
	}
}
