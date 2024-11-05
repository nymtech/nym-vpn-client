package net.nymtech.nymvpn.service.gateway

import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.withContext
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.nymvpn.module.qualifiers.IoDispatcher
import net.nymtech.vpn.NymApi
import net.nymtech.vpn.model.Country
import nym_vpn_lib.GatewayType
import timber.log.Timber
import javax.inject.Inject

class GatewayLibService @Inject constructor(
	private val nymApi: NymApi,
	private val settingsRepository: SettingsRepository,
	@IoDispatcher private val ioDispatcher: CoroutineDispatcher,
) : GatewayService {

	override suspend fun getCountries(type: GatewayType): Result<Set<Country>> {
		return runCatching {
			withContext(ioDispatcher) {
				val env = settingsRepository.getEnvironment()
				Timber.d("Getting countries from lib api")
				nymApi.gateways(type, env)
			}
		}
	}
}
