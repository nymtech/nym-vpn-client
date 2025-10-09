package net.nymtech.vpn.backend

import net.nymtech.vpn.model.NymGateway
import nym_vpn_lib_types.ParsedAccountLinks
import nym_vpn_lib_types.GatewayType
import nym_vpn_lib_types.Network
import nym_vpn_lib_types.SystemMessage
import nym_vpn_lib_types.UserAgent

interface Backend {

	suspend fun getAccountLinks(): ParsedAccountLinks

	suspend fun getSystemMessages(): List<SystemMessage>

	suspend fun getGateways(type: GatewayType): List<NymGateway>

	suspend fun storeMnemonic(credential: String)

	suspend fun isMnemonicStored(): Boolean

	suspend fun isClientNetworkCompatible(appVersion: String): Boolean

	suspend fun getDeviceIdentity(): String

	suspend fun getAccountIdentity(): String

	suspend fun removeMnemonic()

	suspend fun getCurrentEnvironment(): Network

	suspend fun start(tunnel: Tunnel, userAgent: UserAgent)

	suspend fun stop()

	fun getState(): Tunnel.State
}
