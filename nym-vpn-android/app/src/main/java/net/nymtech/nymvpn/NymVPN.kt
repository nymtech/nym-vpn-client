package net.nymtech.nymvpn

import android.app.Application
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.TextUnit
import dagger.hilt.android.HiltAndroidApp
import io.sentry.Sentry
import io.sentry.SentryOptions
import net.nymtech.nymvpn.util.Constants
import net.nymtech.nymvpn.util.navigationBarHeight
import timber.log.Timber


@HiltAndroidApp
class NymVPN : Application() {

    override fun onCreate() {
        super.onCreate()
        instance = this
        if (BuildConfig.DEBUG) Timber.plant(Timber.DebugTree())
        //set lib env vars
        Constants.setupEnvironment()
    }

    companion object {
        lateinit var instance : NymVPN
            private set

        private const val BASELINE_HEIGHT = 2201
        private const val BASELINE_WIDTH = 1080
        private const val BASELINE_DENSITY = 2.625
        fun resizeHeight(dp : Dp) : Dp {
            val displayMetrics = instance.resources.displayMetrics
            val density = displayMetrics.density
            val height = displayMetrics.heightPixels - instance.navigationBarHeight
            val resizeHeightPercentage = (height.toFloat() / BASELINE_HEIGHT) * (BASELINE_DENSITY.toFloat() / density)
            return dp * resizeHeightPercentage
        }

        fun resizeHeight(textUnit: TextUnit) : TextUnit {
            val displayMetrics = instance.resources.displayMetrics
            val density = displayMetrics.density
            val height = displayMetrics.heightPixels - instance.navigationBarHeight
            val resizeHeightPercentage = (height.toFloat() / BASELINE_HEIGHT) * (BASELINE_DENSITY.toFloat() / density)
            return textUnit * resizeHeightPercentage * 1.1
        }

        fun resizeWidth(dp : Dp) : Dp {
            val displayMetrics = instance.resources.displayMetrics
            val density = displayMetrics.density
            val width = displayMetrics.widthPixels
            val resizeWidthPercentage = (width.toFloat() / BASELINE_WIDTH) * (BASELINE_DENSITY.toFloat() / density)
            return dp * resizeWidthPercentage
        }
    }
}