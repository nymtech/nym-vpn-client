package net.nymtech.nymvpn.ui.screens.account.pairing.components

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.semantics.LiveRegionMode
import androidx.compose.ui.semantics.liveRegion
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.ui.common.buttons.MainStyledButton
import net.nymtech.nymvpn.util.extensions.scaledHeight

@Composable
fun PairingStatus(title: String, modifier: Modifier = Modifier, body: String? = null, loading: Boolean = false, actionLabel: String? = null, onAction: () -> Unit = {}) {
	Column(
		modifier = modifier.fillMaxWidth(),
		horizontalAlignment = Alignment.CenterHorizontally,
		verticalArrangement = Arrangement.spacedBy(16.dp),
	) {
		if (loading) CircularProgressIndicator(color = MaterialTheme.colorScheme.primary)
		Text(
			text = title,
			style = MaterialTheme.typography.titleMedium,
			color = MaterialTheme.colorScheme.onBackground,
			textAlign = TextAlign.Center,
			modifier = Modifier.semantics { liveRegion = LiveRegionMode.Polite },
		)
		if (body != null) {
			Text(
				text = body,
				style = MaterialTheme.typography.bodyMedium,
				color = MaterialTheme.colorScheme.outline,
				textAlign = TextAlign.Center,
			)
		}
		if (actionLabel != null) {
			PairingButton(text = actionLabel, onClick = onAction)
		}
	}
}

@Composable
fun PairingButton(text: String, onClick: () -> Unit, modifier: Modifier = Modifier) {
	MainStyledButton(
		onClick = onClick,
		content = { Text(text, style = MaterialTheme.typography.titleMedium) },
		modifier = modifier
			.fillMaxWidth()
			.height(48.dp.scaledHeight()),
		shape = RoundedCornerShape(12.dp),
	)
}
