package net.nymtech.nymvpn

import android.app.Application
import android.os.Build
import android.os.StrictMode
import androidx.lifecycle.DefaultLifecycleObserver
import androidx.lifecycle.LifecycleOwner
import androidx.lifecycle.ProcessLifecycleOwner
import dagger.hilt.android.HiltAndroidApp
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext
import net.nymtech.logcatutil.LogReader
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.nymvpn.module.qualifiers.ApplicationScope
import net.nymtech.nymvpn.module.qualifiers.IoDispatcher
import net.nymtech.nymvpn.module.qualifiers.MainDispatcher
import net.nymtech.nymvpn.util.LocaleUtil
import net.nymtech.nymvpn.util.extensions.requestTileServiceStateUpdate
import net.nymtech.nymvpn.util.timber.ReleaseTree
import net.nymtech.vpn.backend.Backend
import timber.log.Timber
import timber.log.Timber.DebugTree
import javax.inject.Inject
import javax.inject.Provider

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
	lateinit var backend: Provider<Backend>

	@Inject
	lateinit var logReader: LogReader

	override fun onCreate() {
		super.onCreate()
		instance = this
		ProcessLifecycleOwner.get().lifecycle.addObserver(AppLifecycleObserver())
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

		logReader.initialize()

		applicationScope.launch {
			settingsRepository.getLocale()?.let {
				withContext(mainDispatcher) {
					LocaleUtil.changeLocale(it)
				}
			}
		}
		requestTileServiceStateUpdate()
	}

	class AppLifecycleObserver : DefaultLifecycleObserver {

		override fun onStart(owner: LifecycleOwner) {
			Timber.d("Application entered foreground")
			foreground = true
		}
		override fun onPause(owner: LifecycleOwner) {
			Timber.d("Application entered background")
			foreground = false
		}
	}

	companion object {
		private var foreground = false

		fun isForeground(): Boolean {
			return foreground
		}

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
