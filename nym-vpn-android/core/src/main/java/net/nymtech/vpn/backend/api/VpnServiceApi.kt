package net.nymtech.vpn.backend.api

import kotlinx.coroutines.flow.Flow
import net.nymtech.vpn.backend.ConnectInitRequest
import net.nymtech.vpn.backend.ConnectRequest
import net.nymtech.vpn.backend.ConnectResult
import net.nymtech.vpn.backend.Tunnel
import net.nymtech.vpn.backend.VpnServiceEvent
import net.nymtech.vpn.model.NymGateway
import nym_vpn_lib_types.AccountControllerState
import nym_vpn_lib_types.FeatureFlags
import nym_vpn_lib_types.GatewayType
import nym_vpn_lib_types.NetworkCompatibility
import nym_vpn_lib_types.ParsedAccountLinks
import nym_vpn_lib_types.SystemMessage

interface VpnServiceApi {

	companion object {
		const val ACTION_BIND_APP = "net.nymtech.vpn.backend.service.BIND_APP"
	}

	suspend fun init(request: ConnectInitRequest): ConnectResult

	fun getState(): Tunnel.State

	suspend fun connect(request: ConnectRequest): ConnectResult
	suspend fun disconnect(): ConnectResult
	val events: Flow<VpnServiceEvent>

	suspend fun isMnemonicStored(): Boolean
	suspend fun storeMnemonic(mnemonic: String)
	suspend fun removeMnemonic()

	suspend fun getAccountState(): AccountControllerState
	suspend fun getAccountLinks(locale: String): ParsedAccountLinks?
	suspend fun getSystemMessages(): List<SystemMessage>

	suspend fun getGateways(type: GatewayType): List<NymGateway>

	suspend fun getNetworkVersions(): NetworkCompatibility?
	suspend fun getDeviceIdentity(): String?
	suspend fun getAccountIdentity(): String?
	suspend fun getFeatureFlags(): FeatureFlags?
}
