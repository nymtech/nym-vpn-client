package net.nymtech.nymvpn.ui.screens.hop.components

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.outlined.OpenInNew
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
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.util.extensions.scaledHeight

@Composable
fun ServerDetailsModalBody(onClick: () -> Unit) {
	Column(verticalArrangement = Arrangement.spacedBy(16.dp.scaledHeight())) {
		Text(
			text = stringResource(R.string.gateway_modal_description),
			style = MaterialTheme.typography.bodyMedium,
			color = MaterialTheme.colorScheme.onSurfaceVariant,
			textAlign = TextAlign.Center,
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
					Icon(icon, stringResource(R.string.go), Modifier.size(16.dp))
				}
			}
		}
	}
}
