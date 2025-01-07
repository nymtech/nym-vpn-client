package net.nymtech.vpn.backend

import android.app.Service
import android.content.Context
import android.content.Intent
import android.os.Build
import android.os.IBinder
import android.os.PowerManager
import android.system.Os
import androidx.core.app.ServiceCompat
import com.getkeepsafe.relinker.ReLinker
import com.getkeepsafe.relinker.ReLinker.LoadListener
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.delay
import kotlinx.coroutines.withContext
import net.nymtech.vpn.model.BackendEvent
import net.nymtech.vpn.model.Country
import net.nymtech.vpn.util.Action
import net.nymtech.vpn.util.Constants
import net.nymtech.vpn.util.Constants.LOG_LEVEL
import net.nymtech.vpn.util.LifecycleVpnService
import net.nymtech.vpn.util.NotificationManager
import net.nymtech.vpn.util.SingletonHolder
import net.nymtech.vpn.util.exceptions.NymVpnInitializeException
import net.nymtech.vpn.util.extensions.asTunnelState
import net.nymtech.vpn.util.extensions.startServiceByClass
import nym_vpn_lib.AccountLinks
import nym_vpn_lib.AccountStateSummary
import nym_vpn_lib.AndroidTunProvider
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
import nym_vpn_lib.isAccountMnemonicStored
import nym_vpn_lib.startVpn
import nym_vpn_lib.stopVpn
import nym_vpn_lib.storeAccountMnemonic
import nym_vpn_lib.waitForRegisterDevice
import nym_vpn_lib.waitForUpdateAccount
import nym_vpn_lib.waitForUpdateDevice
import timber.log.Timber
import java.util.concurrent.TimeoutException
import java.util.concurrent.atomic.AtomicBoolean
import kotlin.properties.Delegates

class NymBackend private constructor(val context: Context) : Backend, TunnelStatusListener {

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
	}

	companion object : SingletonHolder<NymBackend, Context>(::NymBackend) {
		private var vpnService = CompletableDeferred<VpnService>()
		private var stateMachineService = CompletableDeferred<StateMachineService>()
		const val DEFAULT_LOCALE = "en"
	}

	private val initialized = AtomicBoolean(false)

	private val ioDispatcher = Dispatchers.IO

	private val storagePath = context.filesDir.absolutePath

	@get:Synchronized @set:Synchronized
	private var tunnel: Tunnel? = null

	@get:Synchronized @set:Synchronized
	private var state: Tunnel.State = Tunnel.State.Down

	override suspend fun init(environment: Tunnel.Environment, credentialMode: Boolean?) {
		return withContext(ioDispatcher) {
			runCatching {
				initEnvironment(environment)
				nym_vpn_lib.configureLib(storagePath, credentialMode)
				initialized.set(true)
			}.onFailure {
				Timber.e(it)
			}
		}
	}

	suspend fun waitForInit() {
		runCatching {
			val startTime = System.currentTimeMillis()
			val timeout = 5000L
			while (System.currentTimeMillis() - startTime < timeout) {
				if (initialized.get()) {
					return
				}
				delay(10)
			}
			throw TimeoutException("Failed to initialize backend")
		}.onFailure {
			Timber.e(it)
		}
	}

	private suspend fun initEnvironment(environment: Tunnel.Environment) {
		withContext(ioDispatcher) {
			runCatching {
				Os.setenv("RUST_LOG", LOG_LEVEL, true)
				initEnvironment(environment.networkName())
			}.onFailure {
				Timber.w("Failed to setup environment, defaulting to bundle mainnet")
				initFallbackMainnetEnvironment()
			}
		}
	}

	@Throws(VpnException::class)
	override suspend fun getAccountSummary(): AccountStateSummary {
		return withContext(ioDispatcher) {
			waitForInit()
			nym_vpn_lib.getAccountState()
		}
	}

	@Throws(VpnException::class)
	override suspend fun getAccountLinks(): AccountLinks {
		return withContext(ioDispatcher) {
			waitForInit()
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
			try {
				waitForInit()
				storeAccountMnemonic(mnemonic)
				waitForUpdateAccount()
				waitForUpdateDevice()
				waitForRegisterDevice()
			} catch (e: VpnException) {
				forgetAccount()
				throw e
			}
		}
	}

	@Throws(VpnException::class)
	override suspend fun isMnemonicStored(): Boolean {
		return withContext(ioDispatcher) {
			waitForInit()
			// TODO temporary until bug is fixed
			delay(2000L)
			isAccountMnemonicStored()
		}
	}

	@Throws(VpnException::class)
	override suspend fun removeMnemonic() {
		withContext(ioDispatcher) {
			waitForInit()
			forgetAccount()
		}
	}

	override suspend fun getGatewayCountries(type: GatewayType, userAgent: UserAgent): List<Country> {
		return withContext(ioDispatcher) {
			nym_vpn_lib.getGatewayCountries(type, userAgent, null).map {
				Country(isoCode = it.twoLetterIsoCountryCode)
			}
		}
	}

	override suspend fun getSystemMessages(): List<SystemMessage> {
		return withContext(ioDispatcher) {
			waitForInit()
			nym_vpn_lib.getSystemMessages()
		}
	}

	@Throws(NymVpnInitializeException::class)
	override suspend fun start(tunnel: Tunnel, background: Boolean, userAgent: UserAgent) {
		withContext(ioDispatcher) {
			waitForInit()
			val state = getState()
			// TODO handle changes to tunnel while tunnel is up in future
			if (state != Tunnel.State.Down) throw NymVpnInitializeException.VpnAlreadyRunning()
			this@NymBackend.tunnel = tunnel
			onStateChange(Tunnel.State.InitializingClient)
			if (android.net.VpnService.prepare(context) != null) throw NymVpnInitializeException.VpnPermissionDenied()
			startVpn(tunnel, background, userAgent)
		}
	}

	private suspend fun startVpn(tunnel: Tunnel, background: Boolean, userAgent: UserAgent) {
		withContext(ioDispatcher) {
			if (!initialized.get()) init(tunnel.environment, tunnel.credentialMode)
			if (!vpnService.isCompleted) context.startServiceByClass(background, VpnService::class.java)
			context.startServiceByClass(background, StateMachineService::class.java)
			val service = vpnService.await()
			val backend = this@NymBackend
			service.setOwner(backend)
			try {
				startVpn(
					VpnConfig(
						tunnel.entryPoint,
						tunnel.exitPoint,
						isTwoHop(tunnel.mode),
						service,
						storagePath,
						backend,
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
			waitForInit()
			runCatching {
				stopVpn()
				vpnService.getCompleted().stopSelf()
				stateMachineService.getCompleted().stopSelf()
			}.onFailure {
				Timber.e(it)
			}
		}
	}

	override fun getState(): Tunnel.State {
		return state
	}

	private fun isTwoHop(mode: Tunnel.Mode): Boolean = when (mode) {
		Tunnel.Mode.TWO_HOP_MIXNET -> true
		else -> false
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

	internal class StateMachineService : Service() {

		private var wakeLock: PowerManager.WakeLock? = null

		companion object {
			private const val FOREGROUND_ID = 223
			const val SYSTEM_EXEMPT_SERVICE_TYPE_ID = 1024
		}

		override fun onCreate() {
			stateMachineService.complete(this)
			ServiceCompat.startForeground(
				this,
				FOREGROUND_ID,
				notificationManager.createStateMachineNotification(),
				SYSTEM_EXEMPT_SERVICE_TYPE_ID,
			)
			initWakeLock()
			super.onCreate()
		}

		override fun onDestroy() {
			stateMachineService = CompletableDeferred()
			wakeLock?.let {
				if (it.isHeld) {
					it.release()
				}
			}
			stopForeground(STOP_FOREGROUND_REMOVE)
			super.onDestroy()
		}

		override fun onBind(p0: Intent?): IBinder? {
			return null
		}

		private val notificationManager = NotificationManager.getInstance(this)

		override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
			stateMachineService.complete(this)
			return super.onStartCommand(intent, flags, startId)
		}

		private fun initWakeLock() {
			wakeLock = (getSystemService(POWER_SERVICE) as PowerManager).run {
				val tag = this.javaClass.name
				newWakeLock(PowerManager.PARTIAL_WAKE_LOCK, "$tag::lock").apply {
					try {
						Timber.i("Initiating wakelock forever.. for now..")
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
		private var startId by Delegates.notNull<Int>()
		private val notificationManager = NotificationManager.getInstance(this)

		companion object {
			private const val VPN_NOTIFICATION_ID = 222
		}

		private val builder: Builder
			get() = Builder()

		override fun onCreate() {
			Timber.d("Vpn service created")
			vpnService.complete(this)
			super.onCreate()
		}

		override fun onDestroy() {
			Timber.d("Vpn service destroyed")
			vpnService = CompletableDeferred()
			stopForeground(STOP_FOREGROUND_REMOVE)
			notificationManager.cancel(VPN_NOTIFICATION_ID)
			super.onDestroy()
		}

		override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
			this.startId = startId
			vpnService.complete(this)
			intent?.let {
				if (it.action == Action.START_FOREGROUND.name) {
					startForeground(startId, notificationManager.createVpnRunningNotification())
				} else {
					notificationManager.notify(notificationManager.createVpnRunningNotification(), VPN_NOTIFICATION_ID)
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
	}
}
