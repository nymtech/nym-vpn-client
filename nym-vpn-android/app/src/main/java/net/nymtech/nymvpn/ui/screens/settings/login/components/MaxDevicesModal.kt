package net.nymtech.nymvpn.ui.screens.settings.login.components

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextAlign
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.common.Modal
import net.nymtech.nymvpn.ui.theme.CustomTypography
import net.nymtech.nymvpn.util.extensions.openWebUrl
import nym_vpn_lib.AccountLinks

@Composable
fun MaxDevicesModal(show: Boolean, accountLinks: AccountLinks?, onDismiss: () -> Unit) {
	val context = LocalContext.current

	Modal(
		show = show,
		onDismiss = onDismiss,
		title = {
			Text(
				text = stringResource(R.string.max_devices_reached_title),
				color = MaterialTheme.colorScheme.onSurface,
				style = CustomTypography.labelHuge,
				textAlign = TextAlign.Center,
			)
		},
		text = {
			CredentialModalBody {
				accountLinks?.signIn?.let {
					context.openWebUrl(it)
					onDismiss()
				}
			}
		},
		confirmButton = {
			Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.Center) {
				TextButton(
					onClick = onDismiss,
					content = {
						Text(
							stringResource(R.string.close),
							style = MaterialTheme.typography.labelLarge,
							color = MaterialTheme.colorScheme.primary,
						)
					},
				)
			}
		},
	)
}
