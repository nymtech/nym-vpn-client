package net.nymtech.vpn.backend

import android.content.Context
import android.content.pm.ServiceInfo
import android.os.Build
import androidx.lifecycle.LifecycleObserver
import androidx.lifecycle.ProcessLifecycleOwner
import androidx.lifecycle.lifecycleScope
import com.getkeepsafe.relinker.ReLinker
import com.getkeepsafe.relinker.ReLinker.LoadListener
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import net.nymtech.connectivity.NetworkConnectivityService
import net.nymtech.connectivity.NetworkStatus
import net.nymtech.vpn.backend.service.StateMachineService
import net.nymtech.vpn.backend.service.VpnService
import net.nymtech.vpn.model.BackendEvent
import net.nymtech.vpn.model.NymGateway
import net.nymtech.vpn.model.SettingsConfig
import net.nymtech.vpn.util.Constants
import net.nymtech.vpn.util.Constants.LOG_LEVEL
import net.nymtech.vpn.util.exceptions.BackendException
import net.nymtech.vpn.util.extensions.asTunnelState
import net.nymtech.vpn.util.extensions.startServiceByClass
import net.nymtech.vpn.util.notifications.VpnNotificationManager
import nym_vpn_lib.AndroidConnectivityMonitor
import nym_vpn_lib_types.AccountLinks
import nym_vpn_lib.ConnectivityObserver
import nym_vpn_lib_types.GatewayType
import nym_vpn_lib.NymVpnLibConfig
import nym_vpn_lib_types.SystemMessage
import nym_vpn_lib_types.TunnelEvent
import nym_vpn_lib.TunnelStatusListener
import nym_vpn_lib_types.UserAgent
import nym_vpn_lib.VpnConfig
import nym_vpn_lib.VpnException
import nym_vpn_lib.forgetAccount
import nym_vpn_lib.getNetworkCompatibilityVersions
import nym_vpn_lib.initEnvironment
import nym_vpn_lib.initFallbackMainnetEnvironment
import nym_vpn_lib.initLogger
import nym_vpn_lib.isAccountMnemonicStored
import nym_vpn_lib.login
import nym_vpn_lib.startVpn
import nym_vpn_lib.stopVpn
import org.semver4j.Semver
import timber.log.Timber
import java.util.Locale

class NymBackend private constructor(private val context: Context) : Backend, TunnelStatusListener, LifecycleObserver, AndroidConnectivityMonitor {

	private val initialized = CompletableDeferred<Unit>()

	private val observers: MutableList<ConnectivityObserver> = mutableListOf()

	private val ioDispatcher = Dispatchers.IO

	private val storagePath = context.filesDir.absolutePath

	private val notificationManager = VpnNotificationManager.getInstance(context)

	private lateinit var settingConfig: NymVpnLibConfig

	init {
		ReLinker.loadLibrary(
			context,
			Constants.NYM_VPN_LIB,
			object : LoadListener {
				override fun success() {
					Timber.i("Successfully loaded native nym library")
				}

				override fun failure(t: Throwable) {
					Timber.e(t)
				}
			},
		)
		ProcessLifecycleOwner.get().lifecycleScope.launch(Dispatchers.Main) {
			ProcessLifecycleOwner.get().lifecycle.addObserver(this@NymBackend)
		}
	}

	companion object {
		internal var vpnService = CompletableDeferred<VpnService>()
		internal var stateMachineService = CompletableDeferred<StateMachineService>()
		const val DEFAULT_LOCALE = "en"
		internal var alwaysOnCallback: (() -> Unit)? = null

		@Volatile
		var instance: Backend? = null

		fun getInstance(context: Context, environment: Tunnel.Environment, config: SettingsConfig, userAgent: UserAgent): Backend {
			return instance ?: synchronized(this) {
				instance ?: NymBackend(context).also {
					instance = it
					it.init(environment, config, userAgent)
				}
			}
		}

		fun setAlwaysOnCallback(alwaysOnCallback: () -> Unit) {
			this.alwaysOnCallback = alwaysOnCallback
		}
	}

	@get:Synchronized
	@set:Synchronized
	internal var tunnel: Tunnel? = null

	@get:Synchronized
	@set:Synchronized
	private var state: Tunnel.State = Tunnel.State.Down

	@get:Synchronized
	@set:Synchronized
	private var networkStatus: NetworkStatus = NetworkStatus.Unknown

	private fun init(environment: Tunnel.Environment, config: SettingsConfig, userAgent: UserAgent) = ProcessLifecycleOwner.get().lifecycleScope.launch(
		ioDispatcher,
	) {
		runCatching {
			startNetworkMonitorJob()
			initLogger(null, LOG_LEVEL, false)
			initEnvironment(environment)
			configureLib(config, userAgent)
			initialized.complete(Unit)
		}.onFailure {
			Timber.e(it)
		}
	}

	private fun startNetworkMonitorJob() = ProcessLifecycleOwner.get().lifecycleScope.launch(ioDispatcher) {
		NetworkConnectivityService(context).networkStatus.collect {
			Timber.d("New network event: $it")
			onNetworkStatusChange(it)
		}
	}

	private fun onNetworkStatusChange(networkStatus: NetworkStatus) {
		this.networkStatus = networkStatus
		updateObservers()
	}

	override fun addConnectivityObserver(observer: ConnectivityObserver) {
		if (!observers.any { it.id() == observer.id() }) {
			observers.add(observer)
			updateObservers()
		}
	}

	override fun removeConnectivityObserver(observer: ConnectivityObserver) {
		observers.removeIf { it.id() == observer.id() }
	}

	private fun updateObservers() {
		val isConnected = when (networkStatus) {
			NetworkStatus.Connected -> true
			NetworkStatus.Disconnected -> false
			NetworkStatus.Unknown -> return
		}
		Timber.d("Updating observers.. isConnected=$isConnected")
		observers.forEach {
			it.onNetworkChange(isConnected)
		}
	}

	private suspend fun initEnvironment(environment: Tunnel.Environment) {
		withContext(ioDispatcher) {
			runCatching {
				initEnvironment(storagePath, environment.networkName())
			}.onFailure {
				Timber.e("Failed to setup environment: $it. Defaulting to bundle mainnet")
				initFallbackMainnetEnvironment()
			}
		}
	}

	private suspend fun configureLib(settings: SettingsConfig, userAgent: UserAgent) {
		withContext(ioDispatcher) {
			settingConfig = NymVpnLibConfig(
				storagePath,
				settings.credentialsMode,
				settings.sentryMonitoringEnabled,
				settings.statisticsEnabled,
				this@NymBackend,
				userAgent,
			)
			nym_vpn_lib.configureLib(settingConfig)
		}
	}

	@Throws(VpnException::class)
	override suspend fun getAccountLinks(): AccountLinks {
		return withContext(ioDispatcher) {
			initialized.await()
			nym_vpn_lib.getAccountLinks(getCurrentLocaleLanguageCode())
		}
	}

	private fun getCurrentLocaleLanguageCode(): String {
		return try {
			Locale.getDefault().language.lowercase()
		} catch (_: Exception) {
			DEFAULT_LOCALE
		}
	}

	@Throws(VpnException::class)
	override suspend fun storeMnemonic(mnemonic: String) {
		withContext(ioDispatcher) {
			initialized.await()
			login(mnemonic)
		}
	}

	@Throws(VpnException::class)
	override suspend fun isMnemonicStored(): Boolean {
		return withContext(ioDispatcher) {
			initialized.await()
			isAccountMnemonicStored()
		}
	}

	override suspend fun isClientNetworkCompatible(appVersion: String): Boolean {
		return withContext(ioDispatcher) {
			// assume compatible
			initialized.await()
			val versions = getNetworkCompatibilityVersions() ?: return@withContext true
			val compatibleVersion = Semver(versions.android)
			val currentVersion = Semver(appVersion)
			if (currentVersion.isGreaterThanOrEqualTo(compatibleVersion)) {
				Timber.d("Client is compatible with current network version")
				return@withContext true
			}
			Timber.d(
				"Client is incompatible with current network version. " +
					"Client: $currentVersion, Network: $compatibleVersion",
			)
			return@withContext false
		}
	}

	override suspend fun getDeviceIdentity(): String {
		return withContext(ioDispatcher) {
			initialized.await()
			nym_vpn_lib.getDeviceIdentity()
		}
	}

	override suspend fun getAccountIdentity(): String {
		return withContext(ioDispatcher) {
			initialized.await()
			nym_vpn_lib.getAccountIdentity()
		}
	}

	@Throws(VpnException::class)
	override suspend fun removeMnemonic() {
		withContext(ioDispatcher) {
			initialized.await()
			forgetAccount()
		}
	}

	override suspend fun getSystemMessages(): List<SystemMessage> {
		return withContext(ioDispatcher) {
			initialized.await()
			nym_vpn_lib.getSystemMessages()
		}
	}

	override suspend fun getGateways(type: GatewayType): List<NymGateway> {
		return withContext(ioDispatcher) {
			initialized.await()
			nym_vpn_lib.getGateways(type).map(NymGateway::from)
		}
	}

	override suspend fun start(tunnel: Tunnel, userAgent: UserAgent) {
		withContext(ioDispatcher) {
			initialized.await()
			val state = getState()
			if (state != Tunnel.State.Down) throw BackendException.VpnAlreadyRunning()
			this@NymBackend.tunnel = tunnel
			onStateChange(Tunnel.State.InitializingClient)
			if (android.net.VpnService.prepare(context) != null) throw BackendException.VpnPermissionDenied()
			startVpn(tunnel, userAgent)
		}
	}

	private suspend fun startServices() {
		if (!vpnService.isCompleted) context.startServiceByClass(VpnService::class.java)
		if (!stateMachineService.isCompleted) context.startServiceByClass(StateMachineService::class.java)
		val vpnService = vpnService.await()
		val stateMachineService = stateMachineService.await()
		vpnService.owner = this
		stateMachineService.owner = this
	}

	private suspend fun startVpn(tunnel: Tunnel, userAgent: UserAgent) {
		withContext(ioDispatcher) {
			startServices()
			ensureNotificationAndStartForeground()
			try {
				startVpn(
					VpnConfig(
						tunnel.entryPoint,
						tunnel.exitPoint,
						tunnel.mode.isTwoHop(),
						vpnService.await(),
						storagePath,
						storagePath,
						this@NymBackend,
						null,
						userAgent,
					),
				)
			} catch (e: VpnException) {
				onStartFailure(e)
			}
		}
	}

	private fun onStartFailure(e: VpnException) {
		Timber.e(e)
		onStateChange(Tunnel.State.Down)
		tunnel?.onBackendEvent(BackendEvent.StartFailure(e))
	}

	@OptIn(ExperimentalCoroutinesApi::class)
	override suspend fun stop() {
		withContext(ioDispatcher) {
			initialized.await()
			runCatching {
				stopVpn()
				vpnService.getCompleted().stopSelf()
				stateMachineService.getCompleted().stopSelf()
			}.onFailure {
				Timber.e(it)
			}
			onStateChange(Tunnel.State.Down)
		}
	}

	val notification = notificationManager.buildVpnNotification(
		getState(),
	)

	private suspend fun ensureNotificationAndStartForeground() {
		val vpn = vpnService.await()

		val initialNotification = notificationManager.buildVpnNotification(
			getState(),
		)
		if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
			vpn.startForeground(
				VpnNotificationManager.VPN_FOREGROUND_ID,
				initialNotification,
				if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.UPSIDE_DOWN_CAKE) {
					ServiceInfo.FOREGROUND_SERVICE_TYPE_SYSTEM_EXEMPTED
				} else {
					0
				},
			)
		} else {
			vpn.startForeground(
				VpnNotificationManager.VPN_FOREGROUND_ID,
				initialNotification,
			)
		}

		notificationManager.withNotificationPermission {
			val updatedNotification = notificationManager.buildVpnNotification(
				getState(),
			)
			if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
				vpn.startForeground(
					VpnNotificationManager.VPN_FOREGROUND_ID,
					updatedNotification,
					if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.UPSIDE_DOWN_CAKE) {
						ServiceInfo.FOREGROUND_SERVICE_TYPE_SYSTEM_EXEMPTED
					} else {
						0
					},
				)
			} else {
				vpn.startForeground(
					VpnNotificationManager.VPN_FOREGROUND_ID,
					updatedNotification,
				)
			}
		}
	}

	override fun getState(): Tunnel.State {
		return state
	}

	override fun onEvent(event: TunnelEvent) {
		when (event) {
			is TunnelEvent.MixnetState -> {
				tunnel?.onBackendEvent(BackendEvent.Mixnet(event.v1))
			}

			is TunnelEvent.NewState -> {
				onStateChange(event.asTunnelState())
				tunnel?.onBackendEvent(BackendEvent.Tunnel(event.v1))
			}
		}
	}

	private fun onStateChange(state: Tunnel.State) {
		this.state = state
		tunnel?.onStateChange(state)

		notificationManager.updateVpnNotification(state)
	}
}
