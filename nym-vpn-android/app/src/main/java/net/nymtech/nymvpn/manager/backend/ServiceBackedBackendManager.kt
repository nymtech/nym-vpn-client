package net.nymtech.nymvpn.manager.backend

import android.content.Context
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.SharingStarted
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.flow.catch
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.flatMapLatest
import kotlinx.coroutines.flow.stateIn
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.launch
import kotlinx.coroutines.plus
import net.nymtech.vpn.model.VpnServiceEvent
import net.nymtech.nymvpn.NymVpn
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.nymvpn.data.SplitTunnelingRepository
import net.nymtech.nymvpn.data.config.VpnConfigRepository
import net.nymtech.nymvpn.di.qualifiers.ApplicationScope
import net.nymtech.nymvpn.di.qualifiers.IoDispatcher
import net.nymtech.nymvpn.manager.backend.model.TunnelManagerState
import net.nymtech.nymvpn.service.notification.NotificationService
import net.nymtech.nymvpn.ui.common.snackbar.SnackbarController
import net.nymtech.nymvpn.util.StringValue
import net.nymtech.vpn.backend.Tunnel
import net.nymtech.vpn.config.CoreVpnConfigUpdate
import net.nymtech.vpn.model.RecentGateways
import net.nymtech.vpn.model.connect.ConnectInitRequest
import net.nymtech.vpn.model.connect.ConnectResult
import nym_vpn_lib_types.AccountControllerState
import nym_vpn_lib_types.AutologinResponse
import nym_vpn_lib_types.DeeplinkClient
import nym_vpn_lib_types.DeeplinkKind
import nym_vpn_lib_types.FeatureFlags
import nym_vpn_lib_types.GatewayType
import nym_vpn_lib_types.GetDeeplinkParams
import nym_vpn_lib_types.StoredAccountMode
import nym_vpn_lib_types.TentativeGateways
import nym_vpn_lib_types.TunnelType
import nym_vpn_lib_types.VpnAccountSummary
import timber.log.Timber
import java.util.Locale
import javax.inject.Inject
import javax.inject.Singleton

@Singleton
class ServiceBackedBackendManager @Inject constructor(
	private val settingsRepository: SettingsRepository,
	private val vpnConfigRepository: VpnConfigRepository,
	private val serviceConnectionManager: VpnServiceConnectionManager,
	private val notificationService: NotificationService,
	private val splitTunnelingRepository: SplitTunnelingRepository,
	@ApplicationContext private val context: Context,
	@ApplicationScope private val applicationScope: CoroutineScope,
	@IoDispatcher private val ioDispatcher: CoroutineDispatcher,
) : BackendManager {

	companion object {
		private const val TAG = "svc-backend-manager"
	}

	private val _state = MutableStateFlow(TunnelManagerState())
	override val stateFlow: StateFlow<TunnelManagerState> =
		_state.stateIn(applicationScope.plus(ioDispatcher), SharingStarted.Eagerly, TunnelManagerState())

	private val _accountSummaryFlow = MutableStateFlow<VpnAccountSummary?>(null)
	override val accountSummaryFlow: StateFlow<VpnAccountSummary?> = _accountSummaryFlow.asStateFlow()

	private val eventReducer = VpnEventReducer(
		context = context,
		state = _state,
	)

	init {
		observeVpnServiceEvents()
	}

	override fun initialize() {
		applicationScope.launch(ioDispatcher) {
			if (_state.value.isInitialized) return@launch

			val api = runCatching { serviceConnectionManager.awaitApi() }
				.getOrElse {
					Timber.tag(TAG).e(it, "VpnServiceApi bind failed")
					_state.update { s -> s.copy(isInitialized = true) }
					return@launch
				}
			runCatching {
				val initReq = buildInitRequest()
				api.init(initReq)
			}.onFailure { Timber.tag(TAG).e(it, "Core init failed") }

			runCatching { vpnConfigRepository.getConfig() }
				.onFailure { Timber.tag(TAG).e(it, "vpnConfigRepository.refresh failed") }

			refreshIdentityState()
			refreshAccountSummary()

			_state.update {
				it.copy(
					isInitialized = true,
					isNetworkCompatible = true,
				)
			}
		}
	}

	override suspend fun startTunnel(relaxGatewayIndependence: Boolean) {
		val restrictedApps = getRestrictedAppsPackages()
		val initReq = buildInitRequest()

		val res = serviceConnectionManager.withApi { api ->
			runCatching {
				api.applyUpdates(listOf(CoreVpnConfigUpdate.SetRestrictedApps(restrictedApps)))
			}.onFailure { t ->
				Timber.tag(TAG).w(t, "apply restricted apps failed")
			}

			runCatching { api.init(initReq) }
				.onFailure { t -> Timber.tag(TAG).w(t, "Auto-init before connect failed") }

			// Must be applied after init(), which force-syncs the persisted config and
			// would otherwise re-enable gateway independence from nodeFamiliesNotificationsEnabled.
			if (relaxGatewayIndependence) {
				runCatching { api.setGatewayIndependenceEnabled(false) }
					.onFailure { Timber.tag(TAG).w(it, "relax gateway independence failed") }
			}

			api.connect()
		}

		when (res) {
			is ConnectResult.Ok -> Timber.tag(TAG).i("StartTunnelSuccess")
			is ConnectResult.PermissionRequired -> notifyVpnPermissionRequired()
			is ConnectResult.Failed -> Timber.tag(TAG).e("StartTunnelFailed %s", res.message)
			is ConnectResult.NotReady -> Timber.tag(TAG).w("StartTunnelNotReady")
		}
	}

	private suspend fun buildInitRequest(): ConnectInitRequest {
		val mixnetParamConfig = getFeatureFlags()?.let {
			settingsRepository.getMixnetTrafficConfig()
		}

		return ConnectInitRequest(
			mixnetParamConfig = mixnetParamConfig,
		)
	}

	override suspend fun stopTunnel() {
		val res = serviceConnectionManager.withApi { it.disconnect() }
		if (res !is ConnectResult.Ok) {
			Timber.tag(TAG).w("StopTunnel result=%s", res::class.java.simpleName)
		}
	}

	override suspend fun requestReconnect(relaxGatewayIndependence: Boolean) {
		val res = serviceConnectionManager.withApi { api ->
			runCatching {
				val restrictedApps = getRestrictedAppsPackages()
				api.applyUpdates(listOf(CoreVpnConfigUpdate.SetRestrictedApps(restrictedApps)))
			}.onFailure { t ->
				Timber.tag(TAG).w(t, "apply restricted apps failed on reconnect")
			}
			if (relaxGatewayIndependence) {
				runCatching { api.setGatewayIndependenceEnabled(false) }
					.onFailure { Timber.tag(TAG).w(it, "relax gateway independence failed on reconnect") }
			}
			api.reconnect()
		}
		if (res !is ConnectResult.Ok) {
			Timber.tag(TAG).w("ReconnectTunnel result=%s", res::class.java.simpleName)
		}
	}

	override suspend fun pushRestrictedApps() {
		val restrictedApps = getRestrictedAppsPackages()
		serviceConnectionManager.withApi { api ->
			runCatching {
				api.applyUpdates(listOf(CoreVpnConfigUpdate.SetRestrictedApps(restrictedApps)))
			}.onFailure { t ->
				Timber.tag(TAG).w(t, "push restricted apps failed")
			}
		}
	}

	override fun getState(): Tunnel.State = serviceConnectionManager.apiFlow.value?.getState() ?: Tunnel.State.Down

	private fun observeVpnServiceEvents() {
		eventReducer.observe(
			scope = applicationScope,
			dispatcher = ioDispatcher,
			apiFlow = serviceConnectionManager.apiFlow,
		)
		applicationScope.launch(ioDispatcher) {
			serviceConnectionManager.apiFlow
				.filterNotNull()
				.flatMapLatest { it.events }
				.catch { t -> Timber.tag(TAG).e(t, "Error in VPN events stream") }
				.collect { event ->
					if (event is VpnServiceEvent.AccountStateChanged) {
						refreshAccountSummary()
					}
				}
		}
	}

	override suspend fun storeMnemonic(mnemonic: String) {
		serviceConnectionManager.withApi { it.storeMnemonic(mnemonic) }
		refreshIdentityState()
	}

	override suspend fun removeMnemonic() {
		serviceConnectionManager.withApi { it.removeMnemonic() }
		refreshIdentityState()
	}

	override suspend fun isMnemonicStored(): Boolean = serviceConnectionManager.withApi { it.isMnemonicStored() }

	override suspend fun getAccountLinks() = serviceConnectionManager.withApi { it.getAccountLinks(Locale.getDefault().language.lowercase()) }

	override suspend fun getAccountState() = serviceConnectionManager.withApi { it.getAccountState() }

	override suspend fun getDeviceId(): String? = serviceConnectionManager.withApi { it.getDeviceIdentity() }

	override suspend fun getAccountId(): String? = serviceConnectionManager.withApi { it.getAccountIdentity() }

	override suspend fun getSystemMessages() = serviceConnectionManager.withApi { it.getSystemMessages() }

	override suspend fun getGateways(gatewayType: GatewayType) = serviceConnectionManager.withApi { it.getGateways(gatewayType) }

	override suspend fun getRecentGateways(tunnelType: TunnelType): RecentGateways? = runCatching {
		serviceConnectionManager.withApi { it.getRecentGateways(tunnelType) }
	}.getOrElse {
		Timber.tag(TAG).w(it, "getRecentGateways failed")
		null
	}

	override suspend fun getMnemonic(): List<String> = serviceConnectionManager.withApi { it.getStoredMnemonic().split(" ") }
	override suspend fun createAccount() {
		serviceConnectionManager.withApi { it.createAccount() }
		refreshIdentityState()
	}
	override suspend fun registerAccount(purchaseToken: String?): String = serviceConnectionManager.withApi { it.registerAccount(purchaseToken) }
	override suspend fun refreshAccount() {
		serviceConnectionManager.withApi { it.refreshAccount() }
		refreshAccountSummary()
	}

	private suspend fun refreshAccountSummary() {
		val summary = runCatching {
			serviceConnectionManager.withApi { it.getAccountSummary() }
		}.getOrElse {
			Timber.tag(TAG).w(it, "refreshAccountSummary failed")
			return
		}
		_accountSummaryFlow.value = summary
	}

	override suspend fun getFeatureFlags(): FeatureFlags? = runCatching {
		serviceConnectionManager.withApi { it.getFeatureFlags() }
	}.getOrElse {
		Timber.e(it, "GetFeatureFlagsFailed")
		null
	}

	override suspend fun getDeeplink(kind: DeeplinkKind): String? {
		val params = GetDeeplinkParams(
			client = DeeplinkClient.MOBILE,
			locale = Locale.getDefault().language.lowercase(),
			kind = kind,
			name = "default",
		)
		return serviceConnectionManager.withApi { it.getDeeplink(params = params) }
	}

	override suspend fun getAutologinDeeplink(kind: DeeplinkKind): AutologinResponse? {
		val params = GetDeeplinkParams(
			client = DeeplinkClient.MOBILE,
			locale = Locale.getDefault().language.lowercase(),
			kind = kind,
			name = "default",
		)
		return serviceConnectionManager.withApi { it.getAutologinDeeplink(params = params) }
	}

	override suspend fun storeDeeplinkAccount(url: String) {
		runCatching {
			serviceConnectionManager.withApi { it.storeDeeplinkAccount(url = url) }
		}.onFailure {
			Timber.tag(TAG).e(it, "Failed to store deeplink account")
		}
		refreshIdentityState()
	}

	override suspend fun getAccountMode(): StoredAccountMode? = serviceConnectionManager.withApi { it.getAccountMode() }

	override suspend fun getAccountSummary(): VpnAccountSummary? = serviceConnectionManager.withApi { it.getAccountSummary() }

	override suspend fun runDiagnostic(): String? = serviceConnectionManager.withApi { it.runDiagnostic() }

	override suspend fun getTentativeGateways(): TentativeGateways? = runCatching {
		serviceConnectionManager.withApi { it.getTentativeGateways() }
	}.getOrElse {
		Timber.tag(TAG).w(it, "getTentativeGateways failed")
		null
	}

	override suspend fun setGatewayIndependenceEnabled(enabled: Boolean) {
		runCatching {
			serviceConnectionManager.withApi { it.setGatewayIndependenceEnabled(enabled) }
		}.onFailure { Timber.tag(TAG).w(it, "setGatewayIndependenceEnabled failed") }
	}

	private fun notifyVpnPermissionRequired() {
		val isAppInForeground = NymVpn.AppLifecycleObserver.isInForeground.value
		if (!isAppInForeground) {
			notificationService.showNotification(
				title = context.getString(R.string.permission_required),
				description = context.getString(R.string.vpn_permission_missing),
			)
		} else {
			SnackbarController.showMessage(StringValue.StringResource(R.string.vpn_permission_missing))
		}
	}

	private suspend fun getRestrictedAppsPackages(): List<String> = splitTunnelingRepository.getAppInfoList()
		.filter { !it.passThroughVpn }
		.map { it.packageName }

	private suspend fun refreshIdentityState() {
		val updateData = runCatching {
			serviceConnectionManager.withApi { api ->
				val stored = api.isMnemonicStored()
				val devId = if (stored) runCatching { api.getDeviceIdentity() }.getOrNull() else null
				val accId = if (stored) runCatching { api.getAccountIdentity() }.getOrNull() else null
				val state = if (stored) runCatching { api.getAccountState() }.getOrNull() else null

				listOf(stored, devId, accId, state)
			}
		}.getOrNull()

		val mnemonicStored = updateData?.get(0) as? Boolean ?: false
		val deviceId = updateData?.get(1) as? String
		val accountId = updateData?.get(2) as? String
		val accountState = updateData?.get(3) as? AccountControllerState

		_state.update {
			it.copy(
				isMnemonicStored = mnemonicStored,
				deviceId = deviceId,
				accountId = accountId,
				accountState = accountState ?: it.accountState,
			)
		}
	}
}
