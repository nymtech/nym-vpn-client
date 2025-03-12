package net.nymtech.nymvpn.ui.screens.main

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.outlined.OpenInNew
import androidx.compose.material.icons.outlined.Speed
import androidx.compose.material.icons.outlined.VisibilityOff
import androidx.compose.material3.Icon
import androidx.compose.material3.LocalMinimumInteractiveComponentSize
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.util.extensions.scaledHeight
import net.nymtech.nymvpn.util.extensions.scaledWidth

@Composable
fun ModeModalBody(onClick: () -> Unit) {
	Column(verticalArrangement = Arrangement.spacedBy(16.dp.scaledHeight())) {
		Row(
			horizontalArrangement = Arrangement.spacedBy(10.dp.scaledWidth(), Alignment.CenterHorizontally),
			verticalAlignment = Alignment.CenterVertically,
		) {
			val icon = Icons.Outlined.Speed
			Icon(icon, icon.name, tint = MaterialTheme.colorScheme.onSurface)
			Text(
				text = stringResource(id = R.string.two_hop_title),
				style = MaterialTheme.typography.labelLarge,
				color = MaterialTheme.colorScheme.onSurface,
			)
		}
		Text(
			text = stringResource(R.string.two_hop_explanation),
			style = MaterialTheme.typography.bodyMedium,
			color = MaterialTheme.colorScheme.onSurfaceVariant,
		)
		Row(
			horizontalArrangement = Arrangement.spacedBy(10.dp.scaledWidth(), Alignment.CenterHorizontally),
			verticalAlignment = Alignment.CenterVertically,
		) {
			val icon = Icons.Outlined.VisibilityOff
			Icon(icon, icon.name, tint = MaterialTheme.colorScheme.onSurface)
			Text(
				text = stringResource(id = R.string.five_hop_mixnet),
				style = MaterialTheme.typography.labelLarge,
				color = MaterialTheme.colorScheme.onSurface,
			)
		}
		Text(
			text = stringResource(R.string.five_hop_explanation),
			style = MaterialTheme.typography.bodyMedium,
			color = MaterialTheme.colorScheme.onSurfaceVariant,
		)
		CompositionLocalProvider(
			LocalMinimumInteractiveComponentSize provides 0.dp,
		) {
			TextButton(
				onClick = {
					onClick()
				},
			) {
				Row(
					modifier = Modifier.fillMaxWidth(),
					horizontalArrangement = Arrangement.spacedBy(2.dp, Alignment.CenterHorizontally),
					verticalAlignment = Alignment.CenterVertically,
				) {
					Text(stringResource(id = R.string.continue_reading), style = MaterialTheme.typography.bodyMedium)
					val icon = Icons.AutoMirrored.Outlined.OpenInNew
					Icon(icon, icon.name, Modifier.size(16.dp))
				}
			}
		}
	}
}
