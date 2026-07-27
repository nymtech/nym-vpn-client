package net.nymtech.vpn.backend.api

import kotlinx.coroutines.flow.Flow
import net.nymtech.vpn.backend.Tunnel
import net.nymtech.vpn.backend.controller.VpnCoreController
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
import nym_vpn_lib_types.DiagnosticRunParams
import nym_vpn_lib_types.FeatureFlags
import nym_vpn_lib_types.GatewayType
import nym_vpn_lib_types.GetDeeplinkParams
import nym_vpn_lib_types.ListGatewaysOptions
import nym_vpn_lib_types.NetworkCompatibility
import nym_vpn_lib_types.ParsedAccountLinks
import nym_vpn_lib_types.RegisterAccountRequest
import nym_vpn_lib_types.StoreAccountRequest
import nym_vpn_lib_types.StoredAccountMode
import nym_vpn_lib_types.SystemMessage
import nym_vpn_lib_types.TentativeGateways
import nym_vpn_lib_types.TunnelType
import nym_vpn_lib_types.VpnAccountSummary
import timber.log.Timber

/**
 * API implementation. Delegates to VpnCoreController.
 */
internal class VpnServiceApiImpl(private val core: VpnCoreController, override val events: Flow<VpnServiceEvent>) : VpnServiceApi {

	companion object {
		private const val TAG = "core-vpn"
	}

	override suspend fun init(request: ConnectInitRequest): ConnectResult = core.init(request)

	override fun getState(): Tunnel.State = core.state

	override suspend fun getConfig(): CoreVpnConfig = core.getConfig()

	override suspend fun applyUpdate(patch: CoreVpnConfigUpdate): ConfigResult = core.applyUpdate(patch)

	override suspend fun applyUpdates(patches: List<CoreVpnConfigUpdate>): ConfigResult = core.applyUpdates(patches)

	override suspend fun connect(): ConnectResult = core.connect()

	override suspend fun disconnect(): ConnectResult = core.disconnect()

	override suspend fun reconnect(): ConnectResult = core.reconnect()

	override suspend fun isMnemonicStored(): Boolean = core.tryWithCoreSender { it.isAccountStored() } ?: false

	override suspend fun storeMnemonic(mnemonic: String) {
		core.requireCoreSender { it.storeAccount(StoreAccountRequest.Vpn(mnemonic)) }
	}

	override suspend fun removeMnemonic() {
		core.requireCoreSender { it.forgetAccount() }
	}

	override suspend fun getStoredMnemonic(): String = core.requireCoreSender { it.getStoredMnemonic() }

	override suspend fun createAccount() {
		Timber.tag(TAG).d("createAccount requested")
		core.requireCoreSender { it.createAccount() }
	}

	override suspend fun registerAccount(token: String?): String {
		Timber.tag(TAG).d("registerAccount requested")
		return core.requireCoreSender { it.registerAccount(RegisterAccountRequest(token)).accountToken }
	}

	override suspend fun refreshAccount() {
		Timber.tag(TAG).d("refreshAccount requested")
		core.requireCoreSender { it.refreshAccount(false) }
	}

	override suspend fun getAccountState(): AccountControllerState = core.requireCoreSender { it.getAccountState() }

	override suspend fun getAccountLinks(locale: String): ParsedAccountLinks? = core.tryWithCoreSender { it.getAccountLinks(locale) }

	override suspend fun getSystemMessages(): List<SystemMessage> = core.tryWithCoreSender { it.getSystemMessages() } ?: emptyList()

	override suspend fun getGateways(type: GatewayType): List<NymGateway> = core.tryWithCoreSender {
		it.listGateways(ListGatewaysOptions(gwType = type, userAgent = null))
			.map(NymGateway.Companion::from)
	} ?: emptyList()

	override suspend fun getRecentGateways(tunnelType: TunnelType): RecentGateways? = core.tryWithCoreSender { sender ->
		val recent = sender.getRecentGateways(tunnelType)
		RecentGateways(
			entry = recent.entry.map(NymGateway.Companion::from),
			exit = recent.exit.map(NymGateway.Companion::from),
		)
	}

	override suspend fun getNetworkVersions(): NetworkCompatibility? = core.tryWithCoreSender { it.getNetworkCompatibility() }

	override suspend fun getDeviceIdentity(): String? = core.tryWithCoreSender { it.getDeviceIdentity() }

	override suspend fun getAccountIdentity(): String? = core.tryWithCoreSender { it.getAccountIdentity() }

	override suspend fun getFeatureFlags(): FeatureFlags? = core.tryWithCoreSender { it.getFeatureFlags() }
	override suspend fun getDeeplink(params: GetDeeplinkParams): String? = core.tryWithCoreSender { it.getDeeplink(params) }
	override suspend fun getAutologinDeeplink(params: GetDeeplinkParams): AutologinResponse? = core.tryWithCoreSender { it.getAutologinDeeplink(params) }

	override suspend fun storeDeeplinkAccount(url: String) {
		core.requireCoreSender { it.deeplinkStoreAccount(url) }
	}

	override suspend fun getAccountMode(): StoredAccountMode? = core.tryWithCoreSender { it.getAccountMode() }
	override suspend fun getAccountSummary(): VpnAccountSummary? = core.tryWithCoreSender { it.getAccountSummary() }
	override suspend fun runDiagnostic(): String? = core.tryWithCoreSender {
		val params = DiagnosticRunParams(null, skipDns = false, skipHttp = false, skipHybridTransport = false)
		it.runDiagnostic(params)
	}

	override suspend fun getTentativeGateways(): TentativeGateways? = core.tryWithCoreSender { it.getTentativeGateways() }

	override suspend fun setGatewayIndependenceEnabled(enabled: Boolean) {
		core.tryWithCoreSender { it.setEnableGatewayIndependence(enabled) }
	}
}
