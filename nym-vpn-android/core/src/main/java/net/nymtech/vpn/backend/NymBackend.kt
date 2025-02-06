package net.nymtech.vpn.backend

import android.content.Context
import android.content.Intent
import android.os.Build
import android.os.PowerManager
import androidx.core.app.ServiceCompat
import androidx.lifecycle.LifecycleObserver
import androidx.lifecycle.LifecycleService
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
import net.nymtech.vpn.model.BackendEvent
import net.nymtech.vpn.model.NymGateway
import net.nymtech.vpn.util.Constants
import net.nymtech.vpn.util.Constants.LOG_LEVEL
import net.nymtech.vpn.util.LifecycleVpnService
import net.nymtech.vpn.util.NotificationManager
import net.nymtech.vpn.util.exceptions.BackendException
import net.nymtech.vpn.util.extensions.asTunnelState
import net.nymtech.vpn.util.extensions.startServiceByClass
import nym_vpn_lib.AccountLinks
import nym_vpn_lib.AccountStateSummary
import nym_vpn_lib.AndroidTunProvider
import nym_vpn_lib.ConnectivityObserver
import nym_vpn_lib.GatewayMinPerformance
import nym_vpn_lib.GatewayType
import nym_vpn_lib.SystemMessage
import nym_vpn_lib.TunnelEvent
import nym_vpn_lib.TunnelNetworkSettings
import nym_vpn_lib.TunnelStatusListener
import nym_vpn_lib.UserAgent
import nym_vpn_lib.VpnConfig
import nym_vpn_lib.VpnException
import nym_vpn_lib.forgetAccount
import nym_vpn_lib.initEnvironment
import nym_vpn_lib.initFallbackMainnetEnvironment
import nym_vpn_lib.initLogger
import nym_vpn_lib.isAccountMnemonicStored
import nym_vpn_lib.login
import nym_vpn_lib.startVpn
import nym_vpn_lib.stopVpn
import timber.log.Timber

class NymBackend private constructor(private val context: Context) : Backend, TunnelStatusListener, LifecycleObserver {

	private val initialized = CompletableDeferred<Unit>()

	private val observers: MutableList<ConnectivityObserver> = mutableListOf()

	private val ioDispatcher = Dispatchers.IO

	private val storagePath = context.filesDir.absolutePath

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
		ProcessLifecycleOwner.get().lifecycle.addObserver(this)
	}

	companion object {
		private var vpnService = CompletableDeferred<VpnService>()
		private var stateMachineService = CompletableDeferred<StateMachineService>()
		const val DEFAULT_LOCALE = "en"
		internal var alwaysOnCallback: (() -> Unit)? = null

		@Volatile
		private var instance: Backend? = null

		fun getInstance(context: Context, environment: Tunnel.Environment, credentialMode: Boolean? = false): Backend {
			return instance ?: synchronized(this) {
				instance ?: NymBackend(context).also {
					instance = it
					it.init(environment, credentialMode)
				}
			}
		}
		fun setAlwaysOnCallback(alwaysOnCallback: () -> Unit) {
			this.alwaysOnCallback = alwaysOnCallback
		}
	}

	@get:Synchronized @set:Synchronized
	private var tunnel: Tunnel? = null

	@get:Synchronized @set:Synchronized
	private var state: Tunnel.State = Tunnel.State.Down

	@get:Synchronized @set:Synchronized
	private var networkStatus: NetworkStatus = NetworkStatus.Unknown

	private fun init(environment: Tunnel.Environment, credentialMode: Boolean?) = ProcessLifecycleOwner.get().lifecycleScope.launch {
		runCatching {
			startNetworkMonitorJob()
			initLogger(null, LOG_LEVEL)
			initEnvironment(environment)
			configureLib(credentialMode)
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

	private fun addConnectivityObserver(observer: ConnectivityObserver) {
		observers.add(observer)
		updateObservers()
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

	private fun removeObserver(observer: ConnectivityObserver) {
		observers.remove(observer)
	}

	private suspend fun initEnvironment(environment: Tunnel.Environment) {
		withContext(ioDispatcher) {
			runCatching {
				initEnvironment(environment.networkName())
			}.onFailure {
				Timber.w("Failed to setup environment, defaulting to bundle mainnet")
				initFallbackMainnetEnvironment()
			}
		}
	}

	private suspend fun configureLib(credentialMode: Boolean?) {
		withContext(ioDispatcher) {
			nym_vpn_lib.configureLib(storagePath, credentialMode)
		}
	}

	@Throws(VpnException::class)
	override suspend fun getAccountSummary(): AccountStateSummary {
		return withContext(ioDispatcher) {
			initialized.await()
			nym_vpn_lib.getAccountState()
		}
	}

	@Throws(VpnException::class)
	override suspend fun getAccountLinks(): AccountLinks {
		return withContext(ioDispatcher) {
			initialized.await()
			nym_vpn_lib.getAccountLinks(getCurrentLocaleCountryCode())
		}
	}

	private fun getCurrentLocaleCountryCode(): String {
// TODO disable for now
// 		return try {
// 			context.resources.configuration.locales.get(0).country.lowercase()
// 		} catch (_: Exception) {
// 			DEFAULT_LOCALE
// 		}
		return DEFAULT_LOCALE
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

	override suspend fun getDeviceIdentity(): String {
		return withContext(ioDispatcher) {
			initialized.await()
			nym_vpn_lib.getDeviceIdentity()
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

	override suspend fun getGateways(type: GatewayType, userAgent: UserAgent): List<NymGateway> {
		return withContext(ioDispatcher) {
			nym_vpn_lib.getGateways(type, userAgent, GatewayMinPerformance(0u, 0u)).map(NymGateway::from)
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
		vpnService.setOwner(this)
		stateMachineService.setOwner(this)
	}

	private suspend fun startVpn(tunnel: Tunnel, userAgent: UserAgent) {
		withContext(ioDispatcher) {
			startServices()
			try {
				startVpn(
					VpnConfig(
						tunnel.entryPoint,
						tunnel.exitPoint,
						tunnel.mode.isTwoHop(),
						vpnService.await(),
						storagePath,
						this@NymBackend,
						tunnel.credentialMode,
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
	}

	internal class StateMachineService : LifecycleService() {

		val notificationManager = NotificationManager.getInstance(this)

		private var owner: NymBackend? = null
		private var wakeLock: PowerManager.WakeLock? = null

		companion object {
			private const val FOREGROUND_ID = 223
			const val SYSTEM_EXEMPT_SERVICE_TYPE_ID = 1024
		}

		fun setOwner(owner: NymBackend?) {
			this.owner = owner
		}

		override fun onCreate() {
			super.onCreate()
			stateMachineService.complete(this)
			ServiceCompat.startForeground(
				this,
				FOREGROUND_ID,
				notificationManager.createStateMachineNotification(),
				SYSTEM_EXEMPT_SERVICE_TYPE_ID,
			)
			initWakeLock()
		}

		override fun onDestroy() {
			stateMachineService = CompletableDeferred()
			wakeLock?.let {
				if (it.isHeld) {
					it.release()
				}
			}
			ServiceCompat.stopForeground(this, ServiceCompat.STOP_FOREGROUND_REMOVE)
			super.onDestroy()
		}

		override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
			stateMachineService.complete(this)
			ServiceCompat.startForeground(
				this,
				FOREGROUND_ID,
				notificationManager.createStateMachineNotification(),
				SYSTEM_EXEMPT_SERVICE_TYPE_ID,
			)
			return super.onStartCommand(intent, flags, startId)
		}

		private fun initWakeLock() {
			wakeLock = (getSystemService(POWER_SERVICE) as PowerManager).run {
				val tag = this.javaClass.name
				newWakeLock(PowerManager.PARTIAL_WAKE_LOCK, "$tag::lock").apply {
					try {
						Timber.d("Initiating wakelock")
						acquire()
					} finally {
						release()
					}
				}
			}
		}
	}

	internal class VpnService : LifecycleVpnService(), AndroidTunProvider {
		private var owner: NymBackend? = null
		private val notificationManager = NotificationManager.getInstance(this)

		private val builder: Builder
			get() = Builder()

		override fun onCreate() {
			super.onCreate()
			Timber.d("Vpn service created")
			vpnService.complete(this)
		}

		override fun onDestroy() {
			Timber.d("Vpn service destroyed")
			vpnService = CompletableDeferred()
			stopForeground(STOP_FOREGROUND_REMOVE)
			super.onDestroy()
		}

		override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
			vpnService.complete(this)
			startForeground(startId, notificationManager.createVpnRunningNotification())
			if (intent == null || intent.component == null || intent.component?.packageName != packageName) {
				Timber.i("Always-on VPN starting tunnel")
				lifecycleScope.launch {
					alwaysOnCallback?.invoke()
				}
			}
			return super.onStartCommand(intent, flags, startId)
		}

		fun setOwner(owner: NymBackend?) {
			this.owner = owner
		}

		override fun bypass(socket: Int) {
			Timber.d("Bypassing socket: $socket")
			protect(socket)
		}

		override fun configureTunnel(config: TunnelNetworkSettings): Int {
			Timber.i("Configuring tunnel")
			if (prepare(this) != null) return -1
			val vpnInterface = builder.apply {
				config.ipv4Settings?.addresses?.forEach {
					Timber.d("Address v4: $it")
					val address = it.split("/")
					addAddress(address.first(), address.last().toInt())
				}
				config.ipv6Settings?.addresses?.forEach {
					Timber.d("Address v6: $it")
					val address = it.split("/")
					addAddress(address.first(), address.last().toInt())
				}
				config.dnsSettings?.servers?.forEach {
					Timber.d("DNS: $it")
					addDnsServer(it)
				}

				config.dnsSettings?.searchDomains?.forEach {
					Timber.d("Adding search domain $it")
					addSearchDomain(it)
				}

				addRoute("0.0.0.0", 0)
				addRoute("::", 0)

				// disable calculated routes for now because we bypass mixnet socket
				// addRoutes(config, calculator)

				setMtu(config.mtu.toInt())

				setBlocking(false)
				if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
					setMetered(false)
				}
			}.establish()
			val fd = vpnInterface?.detachFd() ?: return -1
			return fd
		}

		override fun addConnectivityObserver(observer: ConnectivityObserver) {
			owner?.addConnectivityObserver(observer)
		}

		override fun removeConnectivityObserver(observer: ConnectivityObserver) {
			owner?.removeObserver(observer)
		}
	}
}
