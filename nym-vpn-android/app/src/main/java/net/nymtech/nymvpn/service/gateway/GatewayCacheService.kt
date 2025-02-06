package net.nymtech.nymvpn.service.gateway

interface GatewayCacheService {
	suspend fun updateExitGatewayCache(): Result<Unit>

	suspend fun updateEntryGatewayCache(): Result<Unit>

	suspend fun updateWgGatewayCache(): Result<Unit>
}
