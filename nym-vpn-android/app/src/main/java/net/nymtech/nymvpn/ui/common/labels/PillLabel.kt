package net.nymtech.nymvpn.ui.common.labels

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.IntrinsicSize
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.window.core.layout.WindowHeightSizeClass
import net.nymtech.nymvpn.ui.MainActivity

@Composable
fun PillLabel(text: String, backgroundColor : Color, textColor : Color) {
    val verticalPadding = when(MainActivity.windowHeightSizeClass) {
        WindowHeightSizeClass.MEDIUM, WindowHeightSizeClass.COMPACT -> 8.dp
        else -> { 16.dp }
    }
    val horizontalPadding = when(MainActivity.windowHeightSizeClass) {
        WindowHeightSizeClass.MEDIUM, WindowHeightSizeClass.COMPACT -> 16.dp
        else -> { 24.dp }
    }
    Text(text, textAlign = TextAlign.Center, color = textColor, modifier = Modifier
        .width(IntrinsicSize.Min)
        .height(when(MainActivity.windowHeightSizeClass) {
            WindowHeightSizeClass.MEDIUM, WindowHeightSizeClass.COMPACT -> 40.dp
            else -> { 56.dp }
        },)
        .background(color = backgroundColor, shape = RoundedCornerShape(size = 50.dp)).padding(vertical = verticalPadding, horizontal = horizontalPadding), style = MaterialTheme.typography.labelLarge)
}