package net.nymtech.vpn.backend

import net.nymtech.vpn.model.NymGateway
import nym_vpn_lib.AccountLinks
import nym_vpn_lib.AccountStateSummary
import nym_vpn_lib.GatewayType
import nym_vpn_lib.SystemMessage
import nym_vpn_lib.UserAgent

interface Backend {

	suspend fun getAccountSummary(): AccountStateSummary

	suspend fun getAccountLinks(): AccountLinks

	suspend fun getSystemMessages(): List<SystemMessage>

	suspend fun getGateways(type: GatewayType, userAgent: UserAgent): List<NymGateway>

	suspend fun storeMnemonic(credential: String)

	suspend fun isMnemonicStored(): Boolean

	suspend fun isClientNetworkCompatible(appVersion: String): Boolean

	suspend fun getDeviceIdentity(): String

	suspend fun getAccountIdentity(): String

	suspend fun removeMnemonic()

	suspend fun start(tunnel: Tunnel, userAgent: UserAgent)

	suspend fun stop()

	fun getState(): Tunnel.State
}
