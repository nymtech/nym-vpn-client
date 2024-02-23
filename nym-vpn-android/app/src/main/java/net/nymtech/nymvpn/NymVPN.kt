package net.nymtech.nymvpn

import android.annotation.SuppressLint
import android.app.Application
import android.content.res.Configuration
import android.system.Os
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.TextUnit
import dagger.hilt.android.HiltAndroidApp
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

        private const val baselineHeight = 2201
        private const val baselineWidth = 1080
        private const val baselineDensity = 2.625
        fun resizeHeight(dp : Dp) : Dp {
            val displayMetrics = instance.resources.displayMetrics
            val density = displayMetrics.density
            val height = displayMetrics.heightPixels - instance.navigationBarHeight
            val resizeHeightPercentage = (height.toFloat() / baselineHeight) * (baselineDensity.toFloat() / density)
            return dp * resizeHeightPercentage
        }

        fun resizeHeight(textUnit: TextUnit) : TextUnit {
            val displayMetrics = instance.resources.displayMetrics
            val density = displayMetrics.density
            val height = displayMetrics.heightPixels - instance.navigationBarHeight
            val resizeHeightPercentage = (height.toFloat() / baselineHeight) * (baselineDensity.toFloat() / density)
            return textUnit * resizeHeightPercentage * 1.1
        }

        fun resizeWidth(dp : Dp) : Dp {
            val displayMetrics = instance.resources.displayMetrics
            val density = displayMetrics.density
            val width = displayMetrics.widthPixels
            val resizeWidthPercentage = (width.toFloat() / baselineWidth) * (baselineDensity.toFloat() / density)
            return dp * resizeWidthPercentage
        }
    }
}