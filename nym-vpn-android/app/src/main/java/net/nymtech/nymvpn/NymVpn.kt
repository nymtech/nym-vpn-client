package net.nymtech.nymvpn

import android.app.Application
import android.os.Build
import android.os.StrictMode
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
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import net.nymtech.logcatutil.LogReader
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.nymvpn.di.qualifiers.ApplicationScope
import net.nymtech.nymvpn.di.qualifiers.IoDispatcher
import net.nymtech.nymvpn.di.qualifiers.MainDispatcher
import net.nymtech.nymvpn.manager.backend.BackendManager
import net.nymtech.nymvpn.util.GraphicsFallback
import net.nymtech.nymvpn.util.LocaleUtil
import net.nymtech.nymvpn.util.extensions.requestTileServiceStateUpdate
import net.nymtech.nymvpn.util.timber.ReleaseTree
import net.nymtech.vpn.backend.NymBackend
import timber.log.Timber
import timber.log.Timber.DebugTree
import javax.inject.Inject

@HiltAndroidApp
class NymVpn : Application() {

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
	lateinit var backendManager: BackendManager

	@Inject
	lateinit var logReader: LogReader

	override fun onCreate() {
		GraphicsFallback.applyIfNeeded()
		super.onCreate()
		instance = this
		AppLifecycleObserver.init()
		if (BuildConfig.DEBUG) {
			Timber.plant(DebugTree())
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
		} else {
			Timber.plant(ReleaseTree())
		}

		logReader.start()
		backendManager.initialize()

		NymBackend.setAlwaysOnCallback {
			applicationScope.launch {
				backendManager.startTunnel()
			}
		}
		applicationScope.launch {
			settingsRepository.getLocale()?.let {
				withContext(mainDispatcher) {
					LocaleUtil.changeLocale(it)
				}
			}
		}
		requestTileServiceStateUpdate()
		applicationScope.launch {
			if (settingsRepository.getSentryMonitoringEnabled()) {
				initSentry()
			}
		}
	}

	private fun initSentry() {
		SentryAndroid.init(this) { options ->
			options.dsn = "https://cf027ef57330e976438c2cbbe1903868@o967446.ingest.us.sentry.io/4506859434082304"
			var sampleRate = 0.1
			var sessionSampleRate = 0.05
			if (BuildConfig.DEBUG) {
				sampleRate = 1.0
				sessionSampleRate = 1.0
			}
			options.sampleRate = sampleRate
			options.profileSessionSampleRate = sessionSampleRate
			options.sessionReplay.onErrorSampleRate = sampleRate
			options.sessionReplay.sessionSampleRate = sessionSampleRate
			// Add a callback that will be used before the event is sent to Sentry.
			// With this callback, you can modify the event or, when returning null, also discard the event.
			options.beforeSend =
				SentryOptions.BeforeSendCallback { event: SentryEvent, hint: Hint ->
					if (SentryLevel.DEBUG == event.level) {
						null
					} else {
						event
					}
				}
		}
	}

	object AppLifecycleObserver : DefaultLifecycleObserver {
		private val _isInForeground = MutableStateFlow(false)
		val isInForeground: StateFlow<Boolean> get() = _isInForeground

		override fun onStart(owner: LifecycleOwner) {
			_isInForeground.value = true
		}

		override fun onStop(owner: LifecycleOwner) {
			_isInForeground.value = false
		}

		fun init() {
			ProcessLifecycleOwner.get().lifecycle.addObserver(this)
		}
	}

	companion object {

		val isInitialized: Boolean get() = ::instance.isInitialized

		lateinit var instance: NymVpn
			private set

		fun getCPUArchitecture(): String {
			return when (Build.SUPPORTED_ABIS.firstOrNull()) {
				"arm64-v8a" -> "ARM64"
				"armeabi-v7a" -> "ARM32"
				"x86_64" -> "x86_64"
				"x86" -> "x86"
				else -> "Unknown"
			}
		}
	}
}
