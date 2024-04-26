package net.nymtech.nymvpn

import android.app.Application
import android.content.ComponentName
import android.os.StrictMode
import android.service.quicksettings.TileService
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.TextUnit
import dagger.hilt.android.HiltAndroidApp
import kotlinx.coroutines.MainScope
import kotlinx.coroutines.cancel
import net.nymtech.nymvpn.service.tile.VpnQuickTile
import net.nymtech.nymvpn.util.actionBarSize
import net.nymtech.nymvpn.util.log.DebugTree
import net.nymtech.nymvpn.util.log.ReleaseTree
import net.nymtech.vpn.model.Environment
import timber.log.Timber

@HiltAndroidApp
class NymVpn : Application() {

	override fun onCreate() {
		super.onCreate()
		instance = this
		if (BuildConfig.DEBUG) {
			Timber.plant(DebugTree())
			StrictMode.enableDefaults()
		} else {
			Timber.plant(ReleaseTree())
		}
	}

	override fun onLowMemory() {
		super.onLowMemory()
		applicationScope.cancel("onLowMemory() called by system")
		applicationScope = MainScope()
	}

	companion object {

		var applicationScope = MainScope()
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
