package net.nymtech.nymvpn.ui.common.labels

import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color
import androidx.window.core.layout.WindowHeightSizeClass
import net.nymtech.nymvpn.ui.MainActivity

@Composable
fun StatusInfoLabel(message: String, textColor: Color) {
    Text(message, color = textColor, style = when(MainActivity.windowHeightSizeClass) {
        WindowHeightSizeClass.MEDIUM, WindowHeightSizeClass.COMPACT -> MaterialTheme.typography.labelMedium
        else -> { MaterialTheme.typography.labelLarge }}
    )
}