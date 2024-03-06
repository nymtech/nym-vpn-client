package net.nymtech.nymvpn.ui.common.labels

import androidx.compose.foundation.layout.IntrinsicSize
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.util.scaledWidth

@Composable
fun StatusInfoLabel(message: String, textColor: Color) {
    Text(message, color = textColor, style = MaterialTheme.typography.labelLarge,
        modifier = Modifier.padding(horizontal = 24.dp.scaledWidth()).height(IntrinsicSize.Min))
}