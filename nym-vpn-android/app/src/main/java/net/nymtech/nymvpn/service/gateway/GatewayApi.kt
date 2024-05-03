package net.nymtech.nymvpn.service.gateway

import net.nymtech.nymvpn.service.gateway.domain.Gateway
import retrofit2.http.GET

interface GatewayApi {
	@GET(BASE_PATH)
	suspend fun getGateways(): List<Gateway>

	@GET("$BASE_PATH/entry")
	suspend fun getEntryGateways(): List<Gateway>

	@GET("$BASE_PATH/exit")
	suspend fun getExitGateways(): List<Gateway>

	@GET("$BASE_PATH/countries")
	suspend fun getAllGatewayTwoCharacterCountryCodes(): List<String>

	@GET("$BASE_PATH/entry/countries")
	suspend fun getAllEntryGatewayTwoCharacterCountryCodes(): List<String>

	@GET("$BASE_PATH/exit/countries")
	suspend fun getAllExitGatewayTwoCharacterCountryCodes(): List<String>

	companion object {
		const val BASE_PATH = "directory/gateways"
	}
}
