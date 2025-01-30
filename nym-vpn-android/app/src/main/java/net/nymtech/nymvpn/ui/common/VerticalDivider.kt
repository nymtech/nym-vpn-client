package net.nymtech.nymvpn.ui.common

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.width
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp

@Composable
fun VerticalDivider(modifier: Modifier = Modifier, color: Color = MaterialTheme.colorScheme.outline, thickness: Dp = 1.dp) {
	Box(
		modifier = modifier
			.fillMaxHeight()
			.width(thickness)
			.background(color = color),
	)
}
