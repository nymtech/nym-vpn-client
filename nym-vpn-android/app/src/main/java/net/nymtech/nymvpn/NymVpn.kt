package net.nymtech.nymvpn

import android.app.Application
import android.content.Context
import android.os.StrictMode
import android.os.StrictMode.ThreadPolicy
import android.os.StrictMode.VmPolicy
import dagger.hilt.android.HiltAndroidApp
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.launch
import net.nymtech.localizationutil.LocaleStorage
import net.nymtech.localizationutil.LocaleUtil
import net.nymtech.logcatutil.LogCollect
import net.nymtech.nymvpn.module.ApplicationScope
import net.nymtech.nymvpn.module.IoDispatcher
import net.nymtech.nymvpn.util.extensions.requestTileServiceStateUpdate
import net.nymtech.nymvpn.util.logging.DebugTree
import net.nymtech.nymvpn.util.logging.ReleaseTree
import net.nymtech.vpn.Tunnel
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
	lateinit var logCollect: LogCollect

	override fun onCreate() {
		super.onCreate()
		instance = this
		if (BuildConfig.DEBUG) {
			Timber.plant(DebugTree())
			val builder = VmPolicy.Builder()
			StrictMode.setThreadPolicy(
				ThreadPolicy.Builder()
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
		applicationScope.launch(ioDispatcher) {
			logCollect.start()
		}
		requestTileServiceStateUpdate()
	}

	override fun attachBaseContext(base: Context) {
		super.attachBaseContext(LocaleUtil.getLocalizedContext(base, LocaleStorage(base).getPreferredLocale()))
	}

	companion object {

		lateinit var instance: NymVpn
			private set

		val environment = Tunnel.Environment.from(BuildConfig.FLAVOR)
	}
}
