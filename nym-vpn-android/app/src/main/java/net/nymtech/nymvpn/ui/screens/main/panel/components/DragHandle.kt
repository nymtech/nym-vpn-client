package net.nymtech.nymvpn.ui.screens.main.panel.components

import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.unit.dp

@Composable
internal fun DragHandle(onClick: () -> Unit, modifier: Modifier = Modifier) {
	Box(
		modifier = modifier
			.fillMaxWidth()
			.clickable(interactionSource = remember { MutableInteractionSource() }, indication = null) { onClick() },
		contentAlignment = Alignment.Center,
	) {
		Box(
			modifier = Modifier
				.size(width = 32.dp, height = 4.dp)
				.clip(RoundedCornerShape(2.dp))
				.background(MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.4f)),
		)
	}
}
