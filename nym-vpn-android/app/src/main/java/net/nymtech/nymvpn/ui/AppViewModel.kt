package net.nymtech.nymvpn.ui

import androidx.lifecycle.ViewModel
import androidx.lifecycle.viewModelScope
import dagger.hilt.android.lifecycle.HiltViewModel
import java.util.concurrent.atomic.AtomicBoolean
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.Job
import kotlinx.coroutines.delay
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.filter
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.flow.map
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import kotlinx.coroutines.withTimeoutOrNull
import net.nymtech.connectivity.NetworkService
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.data.GatewayRepository
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.nymvpn.data.config.VpnConfigRepository
import net.nymtech.nymvpn.manager.backend.BackendManager
import net.nymtech.nymvpn.service.gateway.GatewayCacheService
import net.nymtech.nymvpn.ui.common.snackbar.SnackbarController
import net.nymtech.nymvpn.ui.screens.account.info.AutologinState
import net.nymtech.nymvpn.util.Constants
import net.nymtech.nymvpn.util.LocaleUtil
import net.nymtech.nymvpn.util.StringValue
import net.nymtech.nymvpn.util.extensions.toSubscriptionUiState
import net.nymtech.vpn.backend.Tunnel
import net.nymtech.vpn.config.CoreVpnConfigUpdate
import nym_vpn_lib_types.AccountControllerState
import nym_vpn_lib_types.DeeplinkKind
import nym_vpn_lib_types.SystemMessage
import timber.log.Timber
import javax.inject.Inject

@HiltViewModel
class AppViewModel
@Inject
constructor(
	private val settingsRepository: SettingsRepository,
	private val vpnConfigRepository: VpnConfigRepository,
	gatewayRepository: GatewayRepository,
	private val gatewayCacheService: GatewayCacheService,
	private val backendManager: BackendManager,
	networkService: NetworkService,
) : ViewModel() {

	companion object {
		private const val TAG = "app-vm"
		private const val ACCOUNT_INIT_TIMEOUT_MS = 30_000L
	}

	private val _systemMessage = MutableStateFlow<SystemMessage?>(null)
	val systemMessage = _systemMessage.asStateFlow()

	private val _configurationChange = MutableStateFlow(false)
	val configurationChange = _configurationChange.asStateFlow()

	private val _isAppReady = MutableStateFlow(false)
	val isAppReady = _isAppReady.asStateFlow()

	private val autoStartAttempted = AtomicBoolean(false)

	private val _autologinState = MutableStateFlow<AutologinState>(AutologinState.Idle)
	val autologinState: StateFlow<AutologinState> = _autologinState.asStateFlow()

	private val accountInitializingState = MutableStateFlow(false)

	private var autologinJob: Job? = null
	private var accountInitJob: Job? = null

	val uiState =
		combine(
			combine(
				settingsRepository.settingsFlow,
				vpnConfigRepository.configFlow,
				backendManager.stateFlow,
				gatewayRepository.gatewayFlow,
				networkService.networkStatus,
			) { settings, config, manager, gateways, networkStatus ->
				AppUiState(
					settings = settings,
					gateways = gateways,
					vpnConfig = config,
					managerState = manager,
					networkStatus = networkStatus,
				)
			},
			backendManager.accountSummaryFlow,
			accountInitializingState,
		) { base, accountSummary, isInitializing ->
			base.copy(
				subscription = accountSummary?.toSubscriptionUiState(),
				isAccountInitializing = isInitializing,
			)
		}.stateIn(
			viewModelScope,
			SharingStarted.WhileSubscribed(Constants.SUBSCRIPTION_TIMEOUT),
			AppUiState(),
		)

	fun fetchAutologin(kind: DeeplinkKind) {
		autologinJob?.cancel()
		autologinJob = viewModelScope.launch {
			_autologinState.value = AutologinState.Loading
			runCatching { backendManager.getAutologinDeeplink(kind) }
				.onSuccess { response ->
					if (response != null) {
						_autologinState.value = AutologinState.PinReady(response.url, response.pinCode)
					} else {
						_autologinState.value = AutologinState.Error(kind)
					}
				}
				.onFailure {
					Timber.tag(TAG).e(it, "autologin failed")
					_autologinState.value = AutologinState.Error(kind)
				}
		}
	}

	fun cancelAutologin() {
		autologinJob?.cancel()
		_autologinState.value = AutologinState.Idle
	}

	fun dismissAutologin() {
		_autologinState.value = AutologinState.Idle
	}

	fun notifyLoginStarted() {
		accountInitJob?.cancel()
		accountInitializingState.value = true
		accountInitJob = viewModelScope.launch {
			withTimeoutOrNull(ACCOUNT_INIT_TIMEOUT_MS) {
				backendManager.stateFlow
					.map { it.accountState }
					.filter { isSettledAccountState(it) }
					.first()
			}
			accountInitializingState.value = false
		}
	}

	private fun isSettledAccountState(state: AccountControllerState): Boolean = state is AccountControllerState.ReadyToConnect ||
		state is AccountControllerState.Decentralised ||
		state is AccountControllerState.UpgradeMode ||
		state is AccountControllerState.PendingSubscription ||
		state is AccountControllerState.Error

	fun onConfigurationHandled() {
		_configurationChange.value = false
	}

	fun logout(onComplete: (() -> Unit)? = null) = viewModelScope.launch(Dispatchers.IO) {
		Timber.tag(TAG).i("LogoutRequested")
		runCatching {
			if (backendManager.getState() != Tunnel.State.Down) {
				Timber.tag(TAG).i("LogoutStoppingTunnel")
				backendManager.stopTunnel()
			}
			performLogout(onComplete)
			Timber.tag(TAG).i("LogoutSuccess")
		}.onFailure {
			Timber.tag(TAG).e(it, "LogoutFailed")
			withContext(Dispatchers.Main) {
				onComplete?.invoke()
			}
		}
	}

	private suspend fun performLogout(onComplete: (() -> Unit)? = null) {
		backendManager.removeMnemonic()
		backendManager.refreshAccount()
		withContext(Dispatchers.Main) {
			onComplete?.invoke()
		}
	}

	fun onLocaleChange(localeTag: String) = viewModelScope.launch {
		Timber.tag(TAG).i("LocaleChangeRequested")
		settingsRepository.setLocale(localeTag)
		LocaleUtil.changeLocale(localeTag)
		_configurationChange.update { true }
		Timber.tag(TAG).i("LocaleChangeApplied")
	}

	fun onEnvironmentChange(environment: Tunnel.Environment) = viewModelScope.launch {
		val tunnelState = backendManager.getState()
		if (tunnelState == Tunnel.State.Down) {
			Timber.tag(TAG).i("EnvironmentChangeApplied env=%s", environment)
			vpnConfigRepository.apply(CoreVpnConfigUpdate.SetNetwork(environment))
			SnackbarController.showMessage(StringValue.StringResource(R.string.app_restart_required))
		} else {
			Timber.tag(TAG).w("EnvironmentChangeRejected reason=tunnel_not_down state=%s", tunnelState)
			SnackbarController.showMessage(StringValue.StringResource(R.string.action_requires_tunnel_down))
		}
	}

	fun onCredentialOverride(value: Boolean?) = viewModelScope.launch {
		val tunnelState = backendManager.getState()
		if (tunnelState != Tunnel.State.Down) {
			Timber.tag(TAG).w("CredentialOverrideRejected reason=tunnel_not_down state=%s", tunnelState)
			return@launch SnackbarController.showMessage(
				StringValue.StringResource(R.string.action_requires_tunnel_down),
			)
		}

		Timber.tag(TAG).i("CredentialOverrideApplied value=%s", value)
		settingsRepository.setCredentialMode(value)
		SnackbarController.showMessage(StringValue.StringResource(R.string.app_restart_required))
	}

	private suspend fun checkSystemMessages() {
		runCatching {
			val messages = backendManager.getSystemMessages()
			val first = messages.firstOrNull()
			if (first != null) {
				_systemMessage.emit(first)
				Timber.tag(TAG).i("SystemMessageReceived present=true")
			} else {
				Timber.tag(TAG).d("SystemMessageReceived present=false")
			}
		}.onFailure {
			Timber.tag(TAG).e(it, "SystemMessageFetchFailed")
		}
	}

	private suspend fun checkAutoStartTunnel() {
		if (!autoStartAttempted.compareAndSet(false, true)) return
		runCatching {
			val enabled = settingsRepository.isAutoStartEnabled()
			Timber.tag(TAG).d("AutoStartCheck enabled=%s", enabled)
			if (!enabled) return

			val managerState = withTimeoutOrNull(15_000) {
				backendManager.stateFlow
					.filter { it.isInitialized }
					.first()
			}

			if (managerState == null) {
				Timber.tag(TAG).w("AutoStartSkipped reason=backend_init_timeout timeoutMs=15000")
				return
			}

			if (!managerState.isMnemonicStored) {
				Timber.tag(TAG).d("AutoStartSkipped reason=mnemonic_missing")
				return
			}

			val tunnelState = backendManager.getState()
			Timber.tag(TAG).d("AutoStartTunnelState state=%s", tunnelState)

			if (tunnelState != Tunnel.State.Down) {
				Timber.tag(TAG).d("AutoStartSkipped reason=tunnel_not_down state=%s", tunnelState)
				return
			}

			Timber.tag(TAG).i("AutoStartStartingTunnel")
			backendManager.startTunnel()
			Timber.tag(TAG).i("AutoStartStartRequested")
		}.onFailure {
			Timber.tag(TAG).e(it, "AutoStartFailed")
		}
	}

	fun onAppStartup() = viewModelScope.launch {
		Timber.tag(TAG).i("AppStartupBegin")

		launch { checkAutoStartTunnel() }

		val theme = settingsRepository.getTheme()
		uiState
			.filter { it.settings.theme != null }
			.first { it.settings.theme == theme }
			.let { _isAppReady.emit(true) }

		Timber.tag(TAG).i("AppReady")

		launch { gatewayCacheService.updateExitGatewayCache() }
		launch { gatewayCacheService.updateEntryGatewayCache() }
		launch { gatewayCacheService.updateWgGatewayCache() }

		launch { checkSystemMessages() }

		launch {
			runCatching {
				backendManager.refreshAccount()
				Timber.tag(TAG).d("AccountRefreshSuccess")
			}.onFailure {
				Timber.tag(TAG).e(it, "AccountRefreshFailed")
			}
		}
	}

	suspend fun isUserLoggedIn(): Boolean = backendManager.isMnemonicStored()

	suspend fun handleDeepLinkAuth(url: String): Route = withContext(Dispatchers.IO) {
		try {
			Timber.tag(TAG).i("DeepLinkAuth started.")
			backendManager.storeDeeplinkAccount(url)

			runCatching { backendManager.refreshAccount() }

			delay(2_000L)
			notifyLoginStarted()

			val shouldShowTechnical = !settingsRepository.isTechnicalOptScreenCompleted()
			if (shouldShowTechnical) Route.Technical else Route.Main()
		} catch (e: Exception) {
			Timber.tag(TAG).e(e, "FailedStoreDeeplink or processing error")
			Route.Main(autoStart = false)
		}
	}
}
