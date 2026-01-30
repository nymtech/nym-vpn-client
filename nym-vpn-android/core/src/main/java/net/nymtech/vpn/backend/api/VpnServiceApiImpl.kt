package net.nymtech.vpn.backend.api

import kotlinx.coroutines.flow.Flow
import net.nymtech.vpn.backend.Tunnel
import net.nymtech.vpn.backend.controller.VpnCoreController
import net.nymtech.vpn.config.CoreVpnConfigUpdate
import net.nymtech.vpn.model.NymGateway
import net.nymtech.vpn.model.VpnServiceEvent
import net.nymtech.vpn.model.config.ConfigResult
import net.nymtech.vpn.model.config.CoreVpnConfig
import net.nymtech.vpn.model.connect.ConnectInitRequest
import net.nymtech.vpn.model.connect.ConnectResult
import nym_vpn_lib_types.AccountControllerState
import nym_vpn_lib_types.FeatureFlags
import nym_vpn_lib_types.GatewayType
import nym_vpn_lib_types.ListGatewaysOptions
import nym_vpn_lib_types.NetworkCompatibility
import nym_vpn_lib_types.ParsedAccountLinks
import nym_vpn_lib_types.StoreAccountRequest
import nym_vpn_lib_types.SystemMessage

/**
 * API implementation. Delegates to VpnCoreController.
 */
internal class VpnServiceApiImpl(
	private val core: VpnCoreController,
	override val events: Flow<VpnServiceEvent>,
) : VpnServiceApi {

	override suspend fun init(request: ConnectInitRequest): ConnectResult = core.init(request)

	override fun getState(): Tunnel.State = core.state

	override suspend fun getConfig(): CoreVpnConfig = core.getConfig()

	override suspend fun applyUpdate(patch: CoreVpnConfigUpdate): ConfigResult = core.applyUpdate(patch)

	override suspend fun applyUpdates(patches: List<CoreVpnConfigUpdate>): ConfigResult = core.applyUpdates(patches)

	override suspend fun connect(): ConnectResult = core.connect()

	override suspend fun disconnect(): ConnectResult = core.disconnect()

	override suspend fun isMnemonicStored(): Boolean = core.tryWithCoreSender { it.isAccountStored() } ?: false

	override suspend fun storeMnemonic(mnemonic: String) {
		core.requireCoreSender { it.storeAccount(StoreAccountRequest.Vpn(mnemonic)) }
	}

	override suspend fun removeMnemonic() {
		core.requireCoreSender { it.forgetAccount() }
	}

	override suspend fun getAccountState(): AccountControllerState = core.requireCoreSender { it.getAccountState() }

	override suspend fun getAccountLinks(locale: String): ParsedAccountLinks? = core.tryWithCoreSender { it.getAccountLinks(locale) }

	override suspend fun getSystemMessages(): List<SystemMessage> = core.tryWithCoreSender { it.getSystemMessages() } ?: emptyList()

	override suspend fun getGateways(type: GatewayType): List<NymGateway> = core.tryWithCoreSender {
		it.listGateways(ListGatewaysOptions(gwType = type, userAgent = null))
			.map(NymGateway.Companion::from)
	} ?: emptyList()

	override suspend fun getNetworkVersions(): NetworkCompatibility? = core.tryWithCoreSender { it.getNetworkCompatibility() }

	override suspend fun getDeviceIdentity(): String? = core.tryWithCoreSender { it.getDeviceIdentity() }

	override suspend fun getAccountIdentity(): String? = core.tryWithCoreSender { it.getAccountIdentity() }

	override suspend fun getFeatureFlags(): FeatureFlags? = core.tryWithCoreSender { it.getFeatureFlags() }
}
