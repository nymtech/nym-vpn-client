package net.nymtech.nymvpn.service.country

interface CountryCacheService {
	suspend fun updateExitGatewayCache(): Result<Unit>

	suspend fun updateEntryGatewayCache(): Result<Unit>

	suspend fun updateWgGatewayCache(): Result<Unit>
}
