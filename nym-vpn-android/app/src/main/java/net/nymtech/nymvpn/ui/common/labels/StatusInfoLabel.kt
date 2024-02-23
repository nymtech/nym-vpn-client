package net.nymtech.nymvpn.ui.common.labels

import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color

@Composable
fun StatusInfoLabel(message: String, textColor: Color) {
    Text(message, color = textColor, style = MaterialTheme.typography.labelLarge
    )
}