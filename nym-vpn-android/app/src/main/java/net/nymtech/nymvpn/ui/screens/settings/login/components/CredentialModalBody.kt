package net.nymtech.nymvpn.ui.screens.settings.login.components

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.material3.LocalMinimumInteractiveComponentSize
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.common.buttons.MainStyledButton
import net.nymtech.nymvpn.util.extensions.scaledHeight

@Composable
fun CredentialModalBody(onClick: () -> Unit) {
	Column(verticalArrangement = Arrangement.spacedBy(16.dp.scaledHeight())) {
		Text(
			text = stringResource(R.string.credential_modal_description),
			style = MaterialTheme.typography.bodyMedium,
			color = MaterialTheme.colorScheme.onSurface,
			textAlign = TextAlign.Center,
		)
		CompositionLocalProvider(
			LocalMinimumInteractiveComponentSize provides 0.dp,
		) {
			MainStyledButton(
				onClick = { onClick() },
				content = {
					Text(stringResource(id = R.string.manage_devices), style = MaterialTheme.typography.labelLarge, color = MaterialTheme.colorScheme.onPrimary)
				},
				modifier = Modifier.fillMaxWidth().height(40.dp.scaledHeight()),
			)
		}
	}
}
