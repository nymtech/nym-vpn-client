package net.nymtech.nymvpn.manager.backend

import kotlinx.coroutines.flow.StateFlow
import net.nymtech.nymvpn.manager.backend.model.TunnelManagerState
import net.nymtech.vpn.backend.Tunnel
import net.nymtech.vpn.model.NymGateway
import net.nymtech.vpn.model.RecentGateways
import nym_vpn_lib_types.AccountControllerState
import nym_vpn_lib_types.AutologinResponse
import nym_vpn_lib_types.DeeplinkKind
import nym_vpn_lib_types.FeatureFlags
import nym_vpn_lib_types.GatewayType
import nym_vpn_lib_types.ParsedAccountLinks
import nym_vpn_lib_types.StoredAccountMode
import nym_vpn_lib_types.SystemMessage
import nym_vpn_lib_types.TentativeGateways
import nym_vpn_lib_types.TunnelType
import nym_vpn_lib_types.VpnAccountSummary

interface BackendManager {

	val stateFlow: StateFlow<TunnelManagerState>
	val accountSummaryFlow: StateFlow<VpnAccountSummary?>

	suspend fun stopTunnel()
	suspend fun startTunnel(relaxGatewayIndependence: Boolean = false)
	suspend fun requestReconnect(relaxGatewayIndependence: Boolean = false)
	suspend fun storeMnemonic(mnemonic: String)
	suspend fun isMnemonicStored(): Boolean
	suspend fun removeMnemonic()
	suspend fun getAccountLinks(): ParsedAccountLinks?
	suspend fun getSystemMessages(): List<SystemMessage>
	suspend fun getGateways(gatewayType: GatewayType): List<NymGateway>
	suspend fun getRecentGateways(tunnelType: TunnelType): RecentGateways?
	suspend fun createAccount()
	suspend fun registerAccount(purchaseToken: String?): String
	suspend fun refreshAccount()
	suspend fun getMnemonic(): List<String>
	suspend fun getAccountState(): AccountControllerState
	fun getState(): Tunnel.State
	fun initialize()
	suspend fun getDeviceId(): String?
	suspend fun getAccountId(): String?
	suspend fun getFeatureFlags(): FeatureFlags?
	suspend fun getDeeplink(kind: DeeplinkKind): String?
	suspend fun getAutologinDeeplink(kind: DeeplinkKind): AutologinResponse?
	suspend fun storeDeeplinkAccount(url: String)
	suspend fun getAccountMode(): StoredAccountMode?
	suspend fun getAccountSummary(): VpnAccountSummary?
	suspend fun runDiagnostic(): String?
	suspend fun getTentativeGateways(): TentativeGateways?
	suspend fun setGatewayIndependenceEnabled(enabled: Boolean)
}
