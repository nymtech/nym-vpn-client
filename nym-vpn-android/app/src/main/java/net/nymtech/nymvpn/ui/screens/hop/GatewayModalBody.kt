package net.nymtech.nymvpn.ui.screens.hop

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.outlined.OpenInNew
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.LocalMinimumInteractiveComponentEnforcement
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

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun GatewayModalBody(onClick: () -> Unit) {
	Column(verticalArrangement = Arrangement.spacedBy(16.dp.scaledHeight())) {
		Text(
			text = stringResource(R.string.gateway_modal_description),
			style = MaterialTheme.typography.bodyMedium,
			color = MaterialTheme.colorScheme.onSurfaceVariant,
			textAlign = TextAlign.Center,
		)
		CompositionLocalProvider(
			LocalMinimumInteractiveComponentEnforcement provides false,
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
