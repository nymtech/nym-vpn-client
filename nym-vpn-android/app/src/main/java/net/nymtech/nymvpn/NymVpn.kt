package net.nymtech.nymvpn

import android.app.Application
import android.content.Context
import android.os.StrictMode
import androidx.lifecycle.DefaultLifecycleObserver
import androidx.lifecycle.LifecycleOwner
import androidx.lifecycle.ProcessLifecycleOwner
import dagger.hilt.android.HiltAndroidApp
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.launch
import net.nymtech.localizationutil.LocaleStorage
import net.nymtech.localizationutil.LocaleUtil
import net.nymtech.logcatutil.LogCollect
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.nymvpn.module.qualifiers.ApplicationScope
import net.nymtech.nymvpn.module.qualifiers.IoDispatcher
import net.nymtech.nymvpn.util.extensions.requestTileServiceStateUpdate
import net.nymtech.nymvpn.util.timber.DebugTree
import net.nymtech.nymvpn.util.timber.ReleaseTree
import net.nymtech.vpn.backend.Tunnel
import timber.log.Timber
import javax.inject.Inject

@HiltAndroidApp
class NymVpn : Application() {

	val localeStorage: LocaleStorage by lazy {
		LocaleStorage(this)
	}

	@Inject
	@ApplicationScope
	lateinit var applicationScope: CoroutineScope

	@Inject
	@IoDispatcher
	lateinit var ioDispatcher: CoroutineDispatcher

	@Inject
	lateinit var settingsRepository: SettingsRepository

	@Inject
	lateinit var logCollect: LogCollect

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
		applicationScope.launch {
			// need to set env early for cache refresh
			val env = settingsRepository.getEnvironment()
			Timber.d("Configuring for env ${env.name}")
			env.setup()
		}
		applicationScope.launch(ioDispatcher) {
			logCollect.start()
		}
		requestTileServiceStateUpdate()
	}

	override fun attachBaseContext(base: Context) {
		super.attachBaseContext(LocaleUtil.getLocalizedContext(base, LocaleStorage(base).getPreferredLocale()))
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

		val environment = Tunnel.Environment.from(BuildConfig.FLAVOR)
	}
}
