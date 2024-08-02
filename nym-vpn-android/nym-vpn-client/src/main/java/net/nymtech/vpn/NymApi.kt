package net.nymtech.vpn

import android.content.Context
import androidx.annotation.RawRes
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.withContext
import kotlinx.serialization.ExperimentalSerializationApi
import kotlinx.serialization.json.Json
import kotlinx.serialization.json.decodeFromStream
import net.nymtech.vpn.model.Country
import net.nymtech.vpn.model.Environment
import nym_vpn_lib.UserAgent
import nym_vpn_lib.getGatewayCountriesUserAgent

class NymApi(
	private val environment: Environment,
	private val ioDispatcher: CoroutineDispatcher,
	private val userAgent: UserAgent,
) {
	suspend fun gateways(exitOnly: Boolean): Set<Country> {
		return withContext(ioDispatcher) {
			getGatewayCountriesUserAgent(environment.apiUrl, environment.explorerUrl, environment.harbourMasterUrl, exitOnly, userAgent).map {
				Country(isoCode = it.twoLetterIsoCountryCode)
			}.toSet()
		}
	}

	suspend fun getLowLatencyEntryCountry(): Country {
		return withContext(ioDispatcher) {
			Country(
				isoCode =
				nym_vpn_lib.getLowLatencyEntryCountry(
					environment.apiUrl,
					environment.explorerUrl,
					environment.harbourMasterUrl,
				).twoLetterIsoCountryCode,
				isLowLatency = true,
			)
		}
	}

	@OptIn(ExperimentalSerializationApi::class)
	private inline fun <reified T> Context.readRawJson(@RawRes rawResId: Int): T {
		resources.openRawResource(rawResId).buffered().use {
			return Json.decodeFromStream<T>(it)
		}
	}
}
