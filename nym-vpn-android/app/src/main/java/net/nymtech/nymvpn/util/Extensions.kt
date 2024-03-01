package net.nymtech.nymvpn.util

import android.annotation.SuppressLint
import android.content.Context
import android.view.WindowInsets
import android.view.WindowManager
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.TextUnit
import net.nymtech.nymvpn.NymVPN

fun Dp.scaledHeight() : Dp {
    return NymVPN.resizeHeight(this)
}

fun Dp.scaledWidth() : Dp {
    return NymVPN.resizeWidth(this)
}

fun TextUnit.scaled() : TextUnit {
    return NymVPN.resizeHeight(this)
}

val Context.navigationBarHeight: Int
    @SuppressLint("NewApi")
    get() {
        val windowManager = getSystemService(Context.WINDOW_SERVICE) as WindowManager
        return windowManager
                .currentWindowMetrics
                .windowInsets
                .getInsets(WindowInsets.Type.navigationBars())
                .bottom
    }