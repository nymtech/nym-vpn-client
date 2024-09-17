package net.nymtech.nymvpn.service.gateway

import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.withContext
import net.nymtech.nymvpn.module.qualifiers.IoDispatcher
import net.nymtech.vpn.model.Country
import nym_vpn_lib.GatewayType
import timber.log.Timber
import javax.inject.Inject

class GatewayApiService @Inject constructor(
	private val gatewayApi: GatewayApi,
	private val gatewayLibService: GatewayLibService,
	@IoDispatcher private val ioDispatcher: CoroutineDispatcher,
) : GatewayService {

	override suspend fun getLowLatencyCountry(): Result<Country> {
		return withContext(ioDispatcher) {
			gatewayLibService.getLowLatencyCountry()
		}
	}

	override suspend fun getCountries(type: GatewayType): Result<Set<Country>> {
		Timber.d("Getting countries from nym api")
		return safeApiCall {
			withContext(ioDispatcher) {
				val countries = when (type) {
					GatewayType.MIXNET_ENTRY -> gatewayApi.getAllEntryGatewayTwoCharacterCountryCodes()
					GatewayType.MIXNET_EXIT -> gatewayApi.getAllExitGatewayTwoCharacterCountryCodes()
					GatewayType.WG -> {
						Timber.w("Not implemented for VPN")
						emptyList()
					}
				}
				countries.map { Country(it) }.toSet()
			}
		}
	}
}
