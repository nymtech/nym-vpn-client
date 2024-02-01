package net.nymtech.nymvpn.ui.theme

import androidx.compose.ui.unit.dp
import androidx.window.core.layout.WindowHeightSizeClass
import net.nymtech.nymvpn.ui.MainActivity

val screenPadding =
    when (MainActivity.windowHeightSizeClass) {
        WindowHeightSizeClass.MEDIUM,
        WindowHeightSizeClass.COMPACT -> 16.dp
        else -> {
            24.dp
        }
    }
