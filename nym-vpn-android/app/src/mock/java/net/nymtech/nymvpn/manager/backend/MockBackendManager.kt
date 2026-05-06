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
import nym_vpn_lib_types.AsnKind
import nym_vpn_lib_types.DeeplinkKind
import nym_vpn_lib_types.FeatureFlags
import nym_vpn_lib_types.GatewayType
import nym_vpn_lib_types.ParsedAccountLinks
import nym_vpn_lib_types.NoHandle
import nym_vpn_lib_types.Score
import nym_vpn_lib_types.StoredAccountMode
import nym_vpn_lib_types.SystemMessage
import nym_vpn_lib_types.FeatureFlags as SdkFeatureFlags
import nym_vpn_lib_types.VpnAccountSummary
import javax.inject.Inject
import javax.inject.Singleton

/**
 * Mock implementation of [BackendManager] for UI testing.
 * Returns canned responses without requiring the real nym-vpnd daemon.
 */
@Singleton
class MockBackendManager @Inject constructor(@ApplicationScope private val applicationScope: CoroutineScope, @IoDispatcher private val ioDispatcher: CoroutineDispatcher) : BackendManager {

	companion object {
		private const val CONNECT_DELAY_MS = 1500L
		private const val DISCONNECT_DELAY_MS = 800L
		private const val SHOW_MOCK_SYSTEM_MESSAGE = false
		private const val MOCK_SYSTEM_MESSAGE_TTL_MS = 60 * 1000L
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

	private val mockEntryGateways = listOf(
		NymGateway(
			identity = "7CWjY3QFoA9dgE535u9bQiXCfzgMZvSpJu842GA1Wn42",
			twoLetterCountryISO = "us",
			description = "Mock entry server in US for UI flow testing.",
			mixnetScore = Score.HIGH,
			wgScore = Score.MEDIUM,
			wgLoad = Score.LOW,
			wgUptime = 98.3f,
			lastUpdated = "2026-03-09T10:00:00Z",
			name = "Mock-Entry-US-01",
			city = "New York",
			region = "New York",
			asn = "AS15169",
			asnName = "Google LLC",
			asnKind = AsnKind.RESIDENTIAL,
			buildVersion = "mock-1.0.0",
			exitIpv4s = listOf("34.117.59.81"),
			exitIpv6s = listOf("2600:1900::1"),
			bridgeInformation = null,
		),
		NymGateway(
			identity = "5mZ8G2XrYF8nW7AqH2kP3DsV9uQ4JrL1tC6bN0eMfYw",
			twoLetterCountryISO = "de",
			description = "Mock entry server in Germany for node selection testing.",
			mixnetScore = Score.HIGH,
			wgScore = Score.HIGH,
			wgLoad = Score.MEDIUM,
			wgUptime = 95.7f,
			lastUpdated = "2026-03-09T10:05:00Z",
			name = "Mock-Entry-DE-01",
			city = "Berlin",
			region = "Berlin",
			asn = "AS3320",
			asnName = "Deutsche Telekom AG",
			asnKind = AsnKind.OTHER,
			buildVersion = "mock-1.0.0",
			exitIpv4s = listOf("91.198.174.192"),
			exitIpv6s = listOf("2a02:26f0::2"),
			bridgeInformation = null,
		),
	)

	private val mockExitGateways = listOf(
		NymGateway(
			identity = "9dP4qW8rM2aX7kJ5bV3tN1fH6uR0cL8sY2eG4pQmTzK",
			twoLetterCountryISO = "gb",
			description = "Mock exit server in UK with stable performance.",
			mixnetScore = Score.MEDIUM,
			wgScore = Score.HIGH,
			wgLoad = Score.LOW,
			wgUptime = 99.1f,
			lastUpdated = "2026-03-09T10:10:00Z",
			name = "Mock-Exit-UK-01",
			city = "London",
			region = "England",
			asn = "AS5607",
			asnName = "Sky UK Limited",
			asnKind = AsnKind.RESIDENTIAL,
			buildVersion = "mock-1.0.0",
			exitIpv4s = listOf("185.199.108.153"),
			exitIpv6s = listOf("2606:50c0::3"),
			bridgeInformation = null,
		),
		NymGateway(
			identity = "3sL7pN9vQ2mX5kR8bT1fH4uD6yC0wE7aJ9zG2qVnMpK",
			twoLetterCountryISO = "jp",
			description = "Mock exit server in Japan for regional selection tests.",
			mixnetScore = Score.HIGH,
			wgScore = Score.MEDIUM,
			wgLoad = Score.MEDIUM,
			wgUptime = 94.2f,
			lastUpdated = "2026-03-09T10:15:00Z",
			name = "Mock-Exit-JP-01",
			city = "Tokyo",
			region = "Kanto",
			asn = "AS2516",
			asnName = "KDDI CORPORATION",
			asnKind = AsnKind.OTHER,
			buildVersion = "mock-1.0.0",
			exitIpv4s = listOf("203.119.101.61"),
			exitIpv6s = listOf("2404:6800::4"),
			bridgeInformation = null,
		),
	)

	private val mockWgGateways = (mockEntryGateways + mockExitGateways).distinctBy { it.identity }
	private val mockFeatureFlags: FeatureFlags =
		object : SdkFeatureFlags(NoHandle) {
			override fun isPrivyEnabled(): Boolean? = true
			override fun isMixnetTuningEnabled(): Boolean? = true
			override fun isDomainFrontingEnabled(): Boolean? = true
			override fun isQuicEnabled(): Boolean? = true
			override fun getGroupFlag(groupName: String, flagName: String): Boolean? = null
		}

	private val mockAccountLinks = ParsedAccountLinks(
		signUp = "https://nym.com/signup",
		signIn = "https://nym.com/signin",
		account = "https://nym.com/account",
	)

	private val mockSystemMessages = if (SHOW_MOCK_SYSTEM_MESSAGE) {
		listOf(
			SystemMessage(
				name = "mock-maintenance",
				displayFrom = System.currentTimeMillis(),
				displayUntil = System.currentTimeMillis() + MOCK_SYSTEM_MESSAGE_TTL_MS,
				message = "Mock mode enabled: backend responses are simulated for UI testing.",
				properties = mapOf("severity" to "info"),
			),
		)
	} else {
		emptyList()
	}

	private val mockAccountSummary = VpnAccountSummary(
		subscriptionValidUntil = null,
		trafficUsedGb = 12uL,
		trafficLimitGb = 50uL,
		trafficResetTime = null,
		accountAddr = "mock-account-address",
		canonicalAccountAddr = "mock-canonical-address",
		authMethods = emptyList(),
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

	override suspend fun getAccountLinks(): ParsedAccountLinks = mockAccountLinks

	override suspend fun getSystemMessages(): List<SystemMessage> {
		val now = System.currentTimeMillis()
		return mockSystemMessages.filter { message ->
			val displayFrom = message.displayFrom
			val displayUntil = message.displayUntil
			val startsOk = displayFrom == null || now >= displayFrom
			val endsOk = displayUntil == null || now <= displayUntil
			startsOk && endsOk
		}
	}

	override suspend fun getGateways(gatewayType: GatewayType): List<NymGateway> = when (gatewayType) {
		GatewayType.MIXNET_ENTRY -> mockEntryGateways
		GatewayType.MIXNET_EXIT -> mockExitGateways
		GatewayType.WG -> mockWgGateways
	}

	override suspend fun createAccount() {
		_state.update { it.copy(isMnemonicStored = true) }
	}

	override suspend fun registerAccount(purchaseToken: String): String = "mock-registration-id"

	override suspend fun refreshAccount() {}

	override suspend fun getMnemonic(): List<String> = listOf("mock", "seed", "phrase", "for", "ui", "testing", "only", "not", "real", "words", "at", "all")

	override suspend fun getAccountState(): AccountControllerState = AccountControllerState.ReadyToConnect

	override suspend fun getDaemonVersion(): String = "mock-1.0.0"

	override suspend fun getDeviceId(): String = "mock-device-id"

	override suspend fun getAccountId(): String = "mock-account-id"

	override suspend fun getFeatureFlags(): FeatureFlags = mockFeatureFlags

	override suspend fun getDeeplink(kind: DeeplinkKind): String = when (kind) {
		DeeplinkKind.PRIVY -> "https://nym.com/privy/auth"
		DeeplinkKind.PRIVY_LINK -> "https://nym.com/privy/link-account"
	}

	override suspend fun storeDeeplinkAccount(url: String) {}

	override suspend fun getAccountMode(): StoredAccountMode = StoredAccountMode.PRIVY

	override suspend fun getAccountSummary(): VpnAccountSummary = mockAccountSummary
}
