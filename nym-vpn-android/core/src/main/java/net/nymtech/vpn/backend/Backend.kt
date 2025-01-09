package net.nymtech.vpn.backend

import net.nymtech.vpn.model.Country
import nym_vpn_lib.AccountLinks
import nym_vpn_lib.AccountStateSummary
import nym_vpn_lib.GatewayType
import nym_vpn_lib.SystemMessage
import nym_vpn_lib.UserAgent

interface Backend {

	suspend fun init(environment: Tunnel.Environment, credentialMode: Boolean?)

	suspend fun getAccountSummary(): AccountStateSummary

	suspend fun getAccountLinks(): AccountLinks

	suspend fun getSystemMessages(): List<SystemMessage>

	suspend fun getGatewayCountries(type: GatewayType, userAgent: UserAgent): List<Country>

	suspend fun storeMnemonic(credential: String)

	suspend fun isMnemonicStored(): Boolean

	suspend fun getDeviceIdentity(): String

	suspend fun removeMnemonic()

	suspend fun start(tunnel: Tunnel, background: Boolean, userAgent: UserAgent)

	suspend fun stop()

	fun getState(): Tunnel.State
}
