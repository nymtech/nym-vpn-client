package net.nymtech.nymvpn.manager.backend

import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.plus
import net.nymtech.nymvpn.di.qualifiers.ApplicationScope
import net.nymtech.nymvpn.di.qualifiers.IoDispatcher
import net.nymtech.nymvpn.manager.backend.model.TunnelManagerState
import net.nymtech.vpn.backend.Tunnel
import net.nymtech.vpn.model.NymGateway
import nym_vpn_lib_types.AccountControllerState
import nym_vpn_lib_types.DeeplinkKind
import nym_vpn_lib_types.FeatureFlags
import nym_vpn_lib_types.GatewayType
import nym_vpn_lib_types.ParsedAccountLinks
import nym_vpn_lib_types.StoredAccountMode
import nym_vpn_lib_types.SystemMessage
import nym_vpn_lib_types.VpnAccountSummary
import javax.inject.Inject
import javax.inject.Singleton

/**
 * Mock implementation of [BackendManager] for UI testing.
 * Returns canned responses without requiring the real nym-vpnd daemon.
 */
@Singleton
class MockBackendManager @Inject constructor(
	@ApplicationScope private val applicationScope: CoroutineScope,
	@IoDispatcher private val ioDispatcher: CoroutineDispatcher,
) : BackendManager {

	companion object {
		private const val CONNECT_DELAY_MS = 1500L
		private const val DISCONNECT_DELAY_MS = 800L
	}

	private val _state = MutableStateFlow(
		TunnelManagerState(
			isInitialized = true,
			isMnemonicStored = true,
			isNetworkCompatible = true,
			tunnelState = Tunnel.State.Down,
			accountState = AccountControllerState.ReadyToConnect,
		),
	)

	override val stateFlow: StateFlow<TunnelManagerState> =
		_state.stateIn(applicationScope.plus(ioDispatcher), SharingStarted.Eagerly, _state.value)

	override fun initialize() {
		_state.update { it.copy(isInitialized = true) }
	}

	override suspend fun startTunnel() {
		_state.update { it.copy(tunnelState = Tunnel.State.EstablishingConnection) }
		delay(CONNECT_DELAY_MS)
		_state.update { it.copy(tunnelState = Tunnel.State.Up) }
	}

	override suspend fun stopTunnel() {
		_state.update { it.copy(tunnelState = Tunnel.State.Disconnecting) }
		delay(DISCONNECT_DELAY_MS)
		_state.update { it.copy(tunnelState = Tunnel.State.Down) }
	}

	override suspend fun requestReconnect() {
		stopTunnel()
		startTunnel()
	}

	override fun getState(): Tunnel.State = _state.value.tunnelState

	override suspend fun storeMnemonic(mnemonic: String) {
		_state.update { it.copy(isMnemonicStored = true) }
	}

	override suspend fun isMnemonicStored(): Boolean = true

	override suspend fun removeMnemonic() {
		_state.update { it.copy(isMnemonicStored = false) }
	}

	override suspend fun getAccountLinks(): ParsedAccountLinks? = null

	override suspend fun getSystemMessages(): List<SystemMessage> = emptyList()

	override suspend fun getGateways(gatewayType: GatewayType): List<NymGateway> = emptyList()

	override suspend fun createAccount() {
		_state.update { it.copy(isMnemonicStored = true) }
	}

	override suspend fun registerAccount(purchaseToken: String): String = "mock-registration-id"

	override suspend fun refreshAccount() {}

	override suspend fun getMnemonic(): List<String> =
		listOf("mock", "seed", "phrase", "for", "ui", "testing", "only", "not", "real", "words", "at", "all")

	override suspend fun getAccountState(): AccountControllerState =
		AccountControllerState.ReadyToConnect

	override suspend fun getDaemonVersion(): String = "mock-1.0.0"

	override suspend fun getDeviceId(): String = "mock-device-id"

	override suspend fun getAccountId(): String = "mock-account-id"

	override suspend fun getFeatureFlags(): FeatureFlags? = null

	override suspend fun getDeeplink(kind: DeeplinkKind): String? = null

	override suspend fun storeDeeplinkAccount(url: String) {}

	override suspend fun getAccountMode(): StoredAccountMode? = null

	override suspend fun getAccountSummary(): VpnAccountSummary? = null
}
