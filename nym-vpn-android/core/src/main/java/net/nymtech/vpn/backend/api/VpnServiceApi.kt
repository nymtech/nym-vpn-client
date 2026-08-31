package net.nymtech.vpn.backend.api

import kotlinx.coroutines.flow.Flow
import net.nymtech.vpn.backend.Tunnel
import net.nymtech.vpn.config.CoreVpnConfigUpdate
import net.nymtech.vpn.model.NymGateway
import net.nymtech.vpn.model.RecentGateways
import net.nymtech.vpn.model.VpnServiceEvent
import net.nymtech.vpn.model.config.ConfigResult
import net.nymtech.vpn.model.config.CoreVpnConfig
import net.nymtech.vpn.model.connect.ConnectInitRequest
import net.nymtech.vpn.model.connect.ConnectResult
import nym_vpn_lib_types.AccountControllerState
import nym_vpn_lib_types.AutologinResponse
import nym_vpn_lib_types.FeatureFlags
import nym_vpn_lib_types.GatewayType
import nym_vpn_lib_types.GetDeeplinkParams
import nym_vpn_lib_types.NetworkCompatibility
import nym_vpn_lib_types.ParsedAccountLinks
import nym_vpn_lib_types.StoredAccountMode
import nym_vpn_lib_types.SystemMessage
import nym_vpn_lib_types.TentativeGateways
import nym_vpn_lib_types.TunnelType
import nym_vpn_lib_types.VpnAccountSummary

interface VpnServiceApi {

	companion object {
		const val ACTION_BIND_APP = "net.nymtech.vpn.backend.service.BIND_APP"
	}

	suspend fun init(request: ConnectInitRequest): ConnectResult

	fun getState(): Tunnel.State

	suspend fun getConfig(): CoreVpnConfig

	suspend fun applyUpdate(patch: CoreVpnConfigUpdate): ConfigResult
	suspend fun applyUpdates(patches: List<CoreVpnConfigUpdate>): ConfigResult

	suspend fun connect(): ConnectResult
	suspend fun disconnect(): ConnectResult
	suspend fun reconnect(): ConnectResult

	val events: Flow<VpnServiceEvent>

	suspend fun isMnemonicStored(): Boolean
	suspend fun storeMnemonic(mnemonic: String)
	suspend fun removeMnemonic()

	suspend fun getStoredMnemonic(): String

	suspend fun createAccount()

	suspend fun registerAccount(token: String?): String
	suspend fun refreshAccount()
	suspend fun getAccountState(): AccountControllerState
	suspend fun getAccountLinks(locale: String): ParsedAccountLinks?
	suspend fun getSystemMessages(): List<SystemMessage>

	suspend fun getGateways(type: GatewayType): List<NymGateway>

	suspend fun getRecentGateways(tunnelType: TunnelType): RecentGateways?

	suspend fun getNetworkVersions(): NetworkCompatibility?
	suspend fun getDeviceIdentity(): String?
	suspend fun getAccountIdentity(): String?
	suspend fun getFeatureFlags(): FeatureFlags?
	suspend fun getDeeplink(params: GetDeeplinkParams): String?
	suspend fun getAutologinDeeplink(params: GetDeeplinkParams): AutologinResponse?
	suspend fun storeDeeplinkAccount(url: String)
	suspend fun getAccountMode(): StoredAccountMode?
	suspend fun getAccountSummary(): VpnAccountSummary?
	suspend fun runDiagnostic(): String?
	suspend fun getTentativeGateways(): TentativeGateways?
	suspend fun setGatewayIndependenceEnabled(enabled: Boolean)
}
