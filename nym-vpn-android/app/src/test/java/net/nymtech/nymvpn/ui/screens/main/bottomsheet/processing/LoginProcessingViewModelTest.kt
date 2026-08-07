package net.nymtech.nymvpn.ui.screens.main.bottomsheet.processing

import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import kotlinx.coroutines.runBlocking
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.nymvpn.data.domain.Settings
import net.nymtech.nymvpn.manager.backend.BackendManager
import net.nymtech.nymvpn.manager.backend.model.TunnelManagerState
import net.nymtech.nymvpn.ui.Route
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.vpn.backend.Tunnel
import net.nymtech.vpn.model.NymGateway
import net.nymtech.vpn.model.RecentGateways
import nym_vpn_lib_types.AccountControllerErrorStateReason
import nym_vpn_lib_types.AccountControllerState
import nym_vpn_lib_types.AutologinResponse
import nym_vpn_lib_types.DeeplinkKind
import nym_vpn_lib_types.FeatureFlags
import nym_vpn_lib_types.GatewayType
import nym_vpn_lib_types.MixnetTrafficConfig
import nym_vpn_lib_types.ParsedAccountLinks
import nym_vpn_lib_types.StoredAccountMode
import nym_vpn_lib_types.SystemMessage
import nym_vpn_lib_types.TentativeGateways
import nym_vpn_lib_types.TunnelType
import nym_vpn_lib_types.VpnAccountSummary
import org.junit.Assert.assertEquals
import org.junit.Assert.assertNull
import org.junit.Assert.assertTrue
import org.junit.Test

class LoginProcessingViewModelTest {

	@Test
	fun credentialsCarouselTick_capsAndResetsDuringSyncing() = runBlocking {
		val viewModel = LoginProcessingViewModel(FakeBackendManager(), FakeSettingsRepository())

		viewModel.runCredentialsCarouselTickOnceForTests(AccountControllerState.Syncing)
		assertEquals(1, viewModel.credentialsCarouselTick.value)

		viewModel.runCredentialsCarouselTickOnceForTests(AccountControllerState.Syncing)
		assertEquals(1, viewModel.credentialsCarouselTick.value)

		viewModel.runCredentialsCarouselTickOnceForTests(AccountControllerState.Syncing, setupCarouselFinished = false)
		assertEquals(0, viewModel.credentialsCarouselTick.value)
	}

	@Test
	fun credentialsCarouselTick_advancesDuringSyncingAfterSetup() = runBlocking {
		val viewModel = LoginProcessingViewModel(FakeBackendManager(), FakeSettingsRepository())

		viewModel.runCredentialsCarouselTickOnceForTests(AccountControllerState.Syncing, setupCarouselFinished = true)
		assertEquals(1, viewModel.credentialsCarouselTick.value)
	}

	@Test
	fun runReadinessWork_maxDeviceReached_failsFast() = runBlocking {
		val backend = FakeBackendManager(
			AccountControllerState.Error(AccountControllerErrorStateReason.MaxDeviceReached),
		)
		val viewModel = LoginProcessingViewModel(backend, FakeSettingsRepository())

		val result = viewModel.runReadinessWorkForTests()

		assertTrue(result is LoginReadinessWorkResult.Failed)
		assertEquals(
			AccountControllerState.Error(AccountControllerErrorStateReason.MaxDeviceReached),
			(result as LoginReadinessWorkResult.Failed).state,
		)
	}

	@Test
	fun runReadinessWork_waitsForReadyToConnect() = runBlocking {
		val backend = FakeBackendManager(AccountControllerState.Syncing)
		val viewModel = LoginProcessingViewModel(backend, FakeSettingsRepository())
		val waiter = launch {
			delay(50)
			backend.emit(AccountControllerState.ReadyToConnect)
		}

		val result = viewModel.runReadinessWorkForTests()
		waiter.join()

		assertTrue(result is LoginReadinessWorkResult.Success)
		assertEquals(AccountControllerState.ReadyToConnect, (result as LoginReadinessWorkResult.Success).state)
	}

	@Test
	fun finishAfterReadiness_failed_setsFailureMessageAndMainRoute() = runBlocking {
		val failedState = AccountControllerState.Error(AccountControllerErrorStateReason.MaxDeviceReached)
		val viewModel = LoginProcessingViewModel(
			FakeBackendManager(failedState),
			FakeSettingsRepository(technicalOptCompleted = true),
		)

		viewModel.finishAfterReadinessWorkForTests(LoginReadinessWorkResult.Failed(failedState))

		assertEquals(R.string.max_devices_reached_title, viewModel.failureMessageRes.value)
		assertEquals(Route.Main(), viewModel.navigationRoute.value)
	}

	@Test
	fun finishAfterReadiness_timedOut_setsTimedOutAndMainRoute() = runBlocking {
		val viewModel = LoginProcessingViewModel(
			FakeBackendManager(),
			FakeSettingsRepository(technicalOptCompleted = true),
		)

		viewModel.finishAfterReadinessWorkForTests(LoginReadinessWorkResult.TimedOut)

		assertTrue(viewModel.timedOut.value)
		assertEquals(Route.Main(), viewModel.navigationRoute.value)
	}

	@Test
	fun finishAfterReadiness_success_welcomesThenRoutesToMain() = runBlocking {
		val viewModel = LoginProcessingViewModel(
			FakeBackendManager(AccountControllerState.ReadyToConnect),
			FakeSettingsRepository(technicalOptCompleted = true),
		)
		val job = launch {
			viewModel.finishAfterReadinessWorkForTests(
				LoginReadinessWorkResult.Success(AccountControllerState.ReadyToConnect),
			)
		}
		delay(100)
		assertEquals(LoginProcessingUiPhase.Welcome, viewModel.uiPhase.value)
		assertNull(viewModel.navigationRoute.value)
		job.join()
		assertEquals(Route.Main(), viewModel.navigationRoute.value)
	}
}

private class FakeBackendManager(initialAccountState: AccountControllerState = AccountControllerState.Syncing) : BackendManager {
	private val managerState = MutableStateFlow(TunnelManagerState(accountState = initialAccountState))
	private val summary = MutableStateFlow<VpnAccountSummary?>(null)

	override val stateFlow = managerState.asStateFlow()
	override val accountSummaryFlow = summary.asStateFlow()

	fun emit(accountState: AccountControllerState) {
		managerState.value = managerState.value.copy(accountState = accountState)
	}

	override suspend fun refreshAccount() = Unit

	override suspend fun stopTunnel() = unsupported()
	override suspend fun startTunnel(relaxGatewayIndependence: Boolean) = unsupported()
	override suspend fun requestReconnect(relaxGatewayIndependence: Boolean) = unsupported()
	override suspend fun storeMnemonic(mnemonic: String) = unsupported()
	override suspend fun isMnemonicStored(): Boolean = false
	override suspend fun removeMnemonic() = unsupported()
	override suspend fun getAccountLinks(): ParsedAccountLinks? = null
	override suspend fun getSystemMessages(): List<SystemMessage> = emptyList()
	override suspend fun getGateways(gatewayType: GatewayType): List<NymGateway> = emptyList()

	override suspend fun getRecentGateways(tunnelType: TunnelType): RecentGateways? = null
	override suspend fun createAccount() = unsupported()
	override suspend fun registerAccount(purchaseToken: String?): String = unsupported()
	override suspend fun getMnemonic(): List<String> = emptyList()
	override suspend fun getAccountState(): AccountControllerState = managerState.value.accountState
	override fun getState(): Tunnel.State = Tunnel.State.Down
	override fun initialize() = Unit
	override suspend fun getDeviceId(): String? = null
	override suspend fun getAccountId(): String? = null
	override suspend fun getFeatureFlags(): FeatureFlags? = null
	override suspend fun getDeeplink(kind: DeeplinkKind): String? = null
	override suspend fun getAutologinDeeplink(kind: DeeplinkKind): AutologinResponse? = null
	override suspend fun storeDeeplinkAccount(url: String) = unsupported()
	override suspend fun getAccountMode(): StoredAccountMode? = null
	override suspend fun getAccountSummary(): VpnAccountSummary? = null
	override suspend fun runDiagnostic(): String? = null
	override suspend fun getTentativeGateways(): TentativeGateways? = null
	override suspend fun setGatewayIndependenceEnabled(enabled: Boolean) = unsupported()

	private fun unsupported(): Nothing = throw UnsupportedOperationException("not used in LoginProcessingViewModelTest")
}

private class FakeSettingsRepository(private val technicalOptCompleted: Boolean = false) : SettingsRepository {
	override suspend fun getTheme(): Theme = Theme.default()
	override suspend fun setTheme(theme: Theme) = Unit
	override suspend fun isAutoStartEnabled(): Boolean = false
	override suspend fun setAutoStart(enabled: Boolean) = Unit
	override suspend fun isApplicationShortcutsEnabled(): Boolean = false
	override suspend fun setApplicationShortcuts(enabled: Boolean) = Unit
	override suspend fun setManualGatewayOverride(enabled: Boolean) = Unit
	override suspend fun setCredentialMode(enabled: Boolean?) = Unit
	override suspend fun isCredentialMode(): Boolean? = null
	override suspend fun getLocale(): String? = null
	override suspend fun setLocale(locale: String) = Unit
	override suspend fun setBatteryDialogSkipped(skip: Boolean) = Unit
	override suspend fun isBatteryDialogSkipped(): Boolean = false
	override suspend fun getStatisticsEnabled(): Boolean = false
	override suspend fun setStatisticsEnabled(enabled: Boolean) = Unit
	override suspend fun isStatsDialogSkipped(): Boolean = false
	override suspend fun setStatsDialogSkipped(skip: Boolean) = Unit
	override suspend fun setTechnicalOptScreenCompleted() = Unit
	override suspend fun isTechnicalOptScreenCompleted(): Boolean = technicalOptCompleted
	override suspend fun getQUICEnabled(): Boolean = false
	override suspend fun setQUICEnabled(enabled: Boolean) = Unit
	override suspend fun getIsStreamServerBannerDisplayed(): Boolean = false
	override suspend fun setIsStreamServerBannerDisplayed(displayed: Boolean) = Unit
	override suspend fun getIsPerAppSecurityBannerDisplayed(): Boolean = false
	override suspend fun setIsPerAppSecurityBannerDisplayed(displayed: Boolean) = Unit
	override suspend fun getLogsEnabled(): Boolean = false
	override suspend fun setLogsEnabled(enabled: Boolean) = Unit
	override suspend fun isWelcomeShown(): Boolean = false
	override suspend fun setWelcomeShown(shown: Boolean) = Unit
	override suspend fun isOnboardingCompleted(): Boolean = false
	override suspend fun setOnboardingCompleted(completed: Boolean) = Unit
	override val settingsFlow: Flow<Settings> = MutableStateFlow(Settings())
	override suspend fun getMixnetTrafficConfig(): MixnetTrafficConfig = unsupported()
	override suspend fun setMixnetTrafficConfig(config: MixnetTrafficConfig) = Unit
	override suspend fun getPanelCollapsed(): Boolean = false
	override suspend fun setPanelCollapsed(collapsed: Boolean) = Unit

	private fun unsupported(): Nothing = throw UnsupportedOperationException("not used in LoginProcessingViewModelTest")
}
