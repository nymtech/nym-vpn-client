package net.nymtech.nymvpn

import android.app.Application
import android.content.ComponentName
import android.os.StrictMode
import android.os.StrictMode.ThreadPolicy
import android.service.quicksettings.TileService
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.TextUnit
import dagger.hilt.android.HiltAndroidApp
import io.sentry.Sentry
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.launch
import net.nymtech.logcathelper.LogCollect
import net.nymtech.logcathelper.model.LogLevel
import net.nymtech.logcathelper.model.LogMessage
import net.nymtech.nymvpn.module.ApplicationScope
import net.nymtech.nymvpn.module.IoDispatcher
import net.nymtech.nymvpn.service.tile.VpnQuickTile
import net.nymtech.nymvpn.util.Constants
import net.nymtech.nymvpn.util.actionBarSize
import net.nymtech.nymvpn.util.log.DebugTree
import net.nymtech.nymvpn.util.log.NymLibException
import net.nymtech.nymvpn.util.log.ReleaseTree
import net.nymtech.vpn.model.Environment
import timber.log.Timber
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
	lateinit var logCollect: LogCollect

	override fun onCreate() {
		super.onCreate()
		instance = this
		if (BuildConfig.DEBUG) {
			Timber.plant(DebugTree())
			StrictMode.setThreadPolicy(
				ThreadPolicy.Builder()
					.detectDiskReads()
					.detectDiskWrites()
					.detectNetwork()
					.penaltyLog()
					.build(),
			)
		} else {
			Timber.plant(ReleaseTree())
		}
		applicationScope.launch(ioDispatcher) {
			logCollect.start(onLogMessage = { reportLibExceptions(it) })
		}
	}

	private fun reportLibExceptions(message: LogMessage) {
		when (message.level) {
			LogLevel.ERROR -> {
				if (message.tag.contains(Constants.NYM_VPN_LIB_TAG)) {
					Sentry.captureException(
						NymLibException("${message.time} - ${message.tag} ${message.message}"),
					)
				}
			}
			else -> Unit
		}
	}

	companion object {

		lateinit var instance: NymVpn
			private set

		val environment = if (BuildConfig.IS_SANDBOX) Environment.SANDBOX else Environment.MAINNET

		private const val BASELINE_HEIGHT = 2201
		private const val BASELINE_WIDTH = 1080
		private const val BASELINE_DENSITY = 2.625

		fun resizeHeight(dp: Dp): Dp {
			val displayMetrics = instance.resources.displayMetrics
			val density = displayMetrics.density
			val height = displayMetrics.heightPixels - instance.actionBarSize
			val resizeHeightPercentage =
				(height.toFloat() / BASELINE_HEIGHT) * (BASELINE_DENSITY.toFloat() / density)
			return dp * resizeHeightPercentage
		}

		fun resizeHeight(textUnit: TextUnit): TextUnit {
			val displayMetrics = instance.resources.displayMetrics
			val density = displayMetrics.density
			val height = displayMetrics.heightPixels - instance.actionBarSize
			val resizeHeightPercentage =
				(height.toFloat() / BASELINE_HEIGHT) * (BASELINE_DENSITY.toFloat() / density)
			return textUnit * resizeHeightPercentage * 1.1
		}

		fun resizeWidth(dp: Dp): Dp {
			val displayMetrics = instance.resources.displayMetrics
			val density = displayMetrics.density
			val width = displayMetrics.widthPixels
			val resizeWidthPercentage =
				(width.toFloat() / BASELINE_WIDTH) * (BASELINE_DENSITY.toFloat() / density)
			return dp * resizeWidthPercentage
		}

		fun requestTileServiceStateUpdate() {
			TileService.requestListeningState(
				instance,
				ComponentName(instance, VpnQuickTile::class.java),
			)
		}
	}
}
