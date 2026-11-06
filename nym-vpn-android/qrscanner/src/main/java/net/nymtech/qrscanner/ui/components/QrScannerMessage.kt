package net.nymtech.qrscanner.ui.components

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.systemBarsPadding
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.Button
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.ShapeDefaults
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp

@Composable
internal fun QrScannerMessage(icon: ImageVector, title: String, body: String, modifier: Modifier = Modifier, actionLabel: String? = null, onAction: () -> Unit = {}) {
	Column(
		modifier = modifier
			.fillMaxSize()
			.systemBarsPadding()
			.padding(horizontal = 24.dp, vertical = 72.dp),
		verticalArrangement = Arrangement.Center,
		horizontalAlignment = Alignment.CenterHorizontally,
	) {
		Box(
			modifier = Modifier
				.size(64.dp)
				.background(MaterialTheme.colorScheme.primary.copy(alpha = 0.15f), CircleShape),
			contentAlignment = Alignment.Center,
		) {
			Icon(imageVector = icon, contentDescription = null, tint = MaterialTheme.colorScheme.primary)
		}
		Spacer(modifier = Modifier.height(24.dp))
		Text(
			text = title,
			style = MaterialTheme.typography.titleLarge,
			color = MaterialTheme.colorScheme.onSurface,
			textAlign = TextAlign.Center,
		)
		Spacer(modifier = Modifier.height(8.dp))
		Text(
			text = body,
			style = MaterialTheme.typography.bodyMedium,
			color = MaterialTheme.colorScheme.onSurfaceVariant,
			textAlign = TextAlign.Center,
		)
		if (actionLabel != null) {
			Spacer(modifier = Modifier.height(32.dp))
			Button(
				onClick = onAction,
				shape = ShapeDefaults.Small,
				modifier = Modifier
					.fillMaxWidth()
					.height(56.dp),
			) {
				Text(text = actionLabel, style = MaterialTheme.typography.labelLarge)
			}
		}
	}
}
