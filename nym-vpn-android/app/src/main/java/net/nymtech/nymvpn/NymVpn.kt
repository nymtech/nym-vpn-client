package net.nymtech.nymvpn

import android.app.ActivityManager
import android.app.Application
import android.content.Context
import android.os.Build
import android.os.StrictMode
import android.util.Log
import androidx.lifecycle.DefaultLifecycleObserver
import androidx.lifecycle.LifecycleOwner
import androidx.lifecycle.ProcessLifecycleOwner
import dagger.hilt.android.HiltAndroidApp
import io.sentry.Hint
import io.sentry.SentryEvent
import io.sentry.SentryLevel
import io.sentry.SentryOptions
import io.sentry.android.core.SentryAndroid
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Job
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.combine
import kotlinx.coroutines.flow.distinctUntilChanged
import kotlinx.coroutines.flow.first
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import kotlinx.coroutines.withTimeoutOrNull
import net.nymtech.connectivity.NetworkService
import net.nymtech.connectivity.NetworkStatus
import net.nymtech.logcatutil.LogReader
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.nymvpn.data.config.VpnConfigRepository
import net.nymtech.nymvpn.di.qualifiers.ApplicationScope
import net.nymtech.nymvpn.di.qualifiers.IoDispatcher
import net.nymtech.nymvpn.di.qualifiers.MainDispatcher
import net.nymtech.nymvpn.manager.backend.BackendManager
import net.nymtech.nymvpn.util.Constants
import net.nymtech.nymvpn.util.ExitReasons
import net.nymtech.nymvpn.util.GraphicsFallback
import net.nymtech.nymvpn.util.LocaleUtil
import net.nymtech.nymvpn.util.extensions.requestTileServiceStateUpdate
import net.nymtech.nymvpn.util.timber.DebugTree
import net.nymtech.nymvpn.util.timber.NoLogTree
import net.nymtech.nymvpn.util.timber.ReleaseTree
import timber.log.Timber
import javax.inject.Inject

object NymVpnLib {
	init {
		System.loadLibrary("nym_vpn_lib_uniffi")
	}
	external fun initContext(context: Context)
}

@HiltAndroidApp
class NymVpn : Application() {

	companion object {
		private const val TAG = "app"
		private const val PRIOR_EXIT_REASONS_MAX = 5

		val isInitialized: Boolean get() = ::instance.isInitialized

		lateinit var instance: NymVpn
			private set

		fun getCPUArchitecture(): String = when (Build.SUPPORTED_ABIS.firstOrNull()) {
			"arm64-v8a" -> "ARM64"
			"armeabi-v7a" -> "ARM32"
			"x86_64" -> "x86_64"
			"x86" -> "x86"
			else -> "Unknown"
		}
	}

	@Inject
	@ApplicationScope
	lateinit var applicationScope: CoroutineScope

	@Inject
	@IoDispatcher
	lateinit var ioDispatcher: CoroutineDispatcher

	@Inject
	@MainDispatcher
	lateinit var mainDispatcher: CoroutineDispatcher

	@Inject
	lateinit var settingsRepository: SettingsRepository

	@Inject
	lateinit var vpnConfigRepository: VpnConfigRepository

	@Inject
	lateinit var backendManager: BackendManager

	@Inject
	lateinit var networkService: NetworkService

	@Inject
	lateinit var logReader: LogReader

	@Volatile
	private var logsEnabled: Boolean = true

	@Volatile
	private var logsDebugEnabled: Boolean = false

	@Volatile
	private var logReaderStarted: Boolean = false

	@Volatile
	private var priorExitReasonsLogged: Boolean = false

	private var logsObserverJob: Job? = null

	override fun onCreate() {
		GraphicsFallback.applyIfNeeded()
		super.onCreate()
		NymVpnLib.initContext(applicationContext)

		instance = this
		AppLifecycleObserver.init()

		Timber.plant(NoLogTree())

		logsObserverJob?.cancel()
		logsObserverJob = applicationScope.launch(ioDispatcher) {
			combine(
				settingsRepository.settingsFlow,
				vpnConfigRepository.configFlow,
			) { settings, coreConfig ->
				settings.logsEnabled to coreConfig.debugLog
			}
				.distinctUntilChanged()
				.collect { (enabled, debugEnabled) ->
					applyLoggingConfig(enabled, debugEnabled)
					if (enabled) {
						ensureLogReaderStarted()
						logPriorExitReasonsOnce()
					}
				}
		}

		applicationScope.launch(ioDispatcher) {
			runCatching {
				awaitValidatedNetworkIfAutoStarting()
				backendManager.initialize()
				Timber.tag(TAG).i("BackendManagerInitializeRequested")
			}.onFailure { t ->
				Timber.tag(TAG).e(t, "BackendManagerInitializeFailed")
			}

			runCatching {
				settingsRepository.getLocale()?.let { localeTag ->
					withContext(mainDispatcher) { LocaleUtil.changeLocale(localeTag) }
					Timber.tag(TAG).i("LocaleApplied")
				}
			}.onFailure { t ->
				Timber.tag(TAG).w(t, "LocaleApplyFailed")
			}

			runCatching {
				requestTileServiceStateUpdate()
				Timber.tag(TAG).d("TileUpdateRequested")
			}.onFailure { t ->
				Timber.tag(TAG).w(t, "TileUpdateRequestFailed")
			}

			runCatching {
				val config = vpnConfigRepository.getConfig()
				val sentryEnabled = config.sentry

				if (sentryEnabled) {
					initSentry()
					Timber.tag(TAG).i("SentryInitRequested")
				} else {
					Timber.tag(TAG).i("SentryInitSkipped reason=disabled")
				}
			}.onFailure { t ->
				Timber.tag(TAG).e(t, "SentryInitFailed")
			}
		}
	}

	private suspend fun awaitValidatedNetworkIfAutoStarting() {
		if (!settingsRepository.isAutoStartEnabled()) return
		val connected = withTimeoutOrNull(Constants.AUTO_START_NETWORK_WAIT_MS) {
			networkService.networkStatus.first { it == NetworkStatus.Connected }
		}
		Timber.tag(TAG).i("AutoStartNetworkAwait connected=%s", connected != null)
	}

	private fun applyLoggingConfig(enabled: Boolean, debugEnabled: Boolean) {
		if (BuildConfig.DEBUG) {
			logsEnabled = true
			logsDebugEnabled = true
		} else {
			logsEnabled = enabled
			logsDebugEnabled = debugEnabled
		}

		Timber.uprootAll()

		if (!logsEnabled) {
			Timber.plant(NoLogTree())
			disableStrictModeLoggingIfNeeded()
			return
		}

		val minPriority = if (logsDebugEnabled) Log.DEBUG else Log.INFO

		if (BuildConfig.DEBUG) {
			Timber.plant(DebugTree(minPriority))
			enableStrictMode()
			Timber.tag(TAG).i("LoggingEnabled build=debug minPriority=$minPriority")
		} else {
			Timber.plant(ReleaseTree(minPriority))
			Timber.tag(TAG).i("LoggingEnabled build=release minPriority=$minPriority")
		}
	}

	private fun enableStrictMode() {
		val builder = StrictMode.VmPolicy.Builder()
		StrictMode.setThreadPolicy(
			StrictMode.ThreadPolicy.Builder()
				.detectDiskReads()
				.detectDiskWrites()
				.detectNetwork()
				.penaltyLog()
				.build(),
		)
		StrictMode.setVmPolicy(builder.build())
	}

	private fun disableStrictModeLoggingIfNeeded() {
		if (!BuildConfig.DEBUG) return
		StrictMode.setThreadPolicy(StrictMode.ThreadPolicy.LAX)
		StrictMode.setVmPolicy(StrictMode.VmPolicy.LAX)
	}

	private suspend fun logPriorExitReasonsOnce() {
		if (priorExitReasonsLogged) return
		priorExitReasonsLogged = true

		if (Build.VERSION.SDK_INT < Build.VERSION_CODES.R) return
		runCatching {
			val activityManager = getSystemService(ACTIVITY_SERVICE) as ActivityManager
			activityManager.getHistoricalProcessExitReasons(packageName, 0, PRIOR_EXIT_REASONS_MAX)
				.forEach { info ->
					// written directly to the log files: lines emitted this early would be
					// wiped by the log reader's logcat -c
					logReader.writeDiagnostic(
						TAG,
						ExitReasons.formatLine(info.timestamp, info.reason, info.status, info.importance, info.description),
					)
				}
		}.onFailure { t ->
			Timber.tag(TAG).w(t, "PriorExitReasonsFailed")
		}
	}

	private fun ensureLogReaderStarted() {
		if (logReaderStarted) return

		runCatching {
			logReader.start()
			logReaderStarted = true
			Timber.tag(TAG).d("LogReaderStarted")
		}.onFailure { t ->
			Timber.tag(TAG).w(t, "LogReaderStartFailed")
		}
	}

	private fun initSentry() {
		SentryAndroid.init(this) { options ->
			options.dsn =
				"https://cf027ef57330e976438c2cbbe1903868@o967446.ingest.us.sentry.io/4506859434082304"

			val sampleRate: Double
			val sessionSampleRate: Double
			if (BuildConfig.DEBUG) {
				sampleRate = 1.0
				sessionSampleRate = 1.0
			} else {
				sampleRate = 0.1
				sessionSampleRate = 0.05
			}

			options.sampleRate = sampleRate
			options.profileSessionSampleRate = sessionSampleRate
			options.sessionReplay.onErrorSampleRate = sampleRate
			options.sessionReplay.sessionSampleRate = sessionSampleRate

			options.beforeSend =
				SentryOptions.BeforeSendCallback { event: SentryEvent, _: Hint ->
					if (SentryLevel.DEBUG == event.level) null else event
				}
		}
	}

	object AppLifecycleObserver : DefaultLifecycleObserver {
		private val _isInForeground = MutableStateFlow(false)
		val isInForeground: StateFlow<Boolean> get() = _isInForeground

		override fun onStart(owner: LifecycleOwner) {
			_isInForeground.value = true
			Timber.tag(TAG).d("ProcessForeground")
		}

		override fun onStop(owner: LifecycleOwner) {
			_isInForeground.value = false
			Timber.tag(TAG).d("ProcessBackground")
		}

		fun init() {
			ProcessLifecycleOwner.get().lifecycle.addObserver(this)
		}
	}
}
