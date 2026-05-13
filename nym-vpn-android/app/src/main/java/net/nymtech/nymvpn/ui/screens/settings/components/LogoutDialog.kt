package net.nymtech.nymvpn.ui.screens.settings.components

import android.content.res.Configuration
import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.Logout
import androidx.compose.material3.CircularProgressIndicator
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.common.Modal
import net.nymtech.nymvpn.ui.common.buttons.MainStyledButton
import net.nymtech.nymvpn.ui.common.buttons.TransparentButton
import net.nymtech.nymvpn.ui.theme.LocalNymColors
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.util.extensions.scaledHeight

@Composable
fun LogoutDialog(show: Boolean, isLoggingOut: Boolean, onDismiss: () -> Unit, onConfirm: () -> Unit) {
	Modal(
		show = show,
		onDismiss = if (isLoggingOut) {
			{}
		} else {
			onDismiss
		},
		icon = Icons.AutoMirrored.Default.Logout,
		title = {
			Text(
				text = stringResource(R.string.log_out_title),
				style = MaterialTheme.typography.titleLarge,
				color = MaterialTheme.colorScheme.onPrimaryContainer,
			)
		},
		text = {
			Text(
				stringResource(R.string.log_out_body),
				textAlign = TextAlign.Center,
				style = MaterialTheme.typography.bodyMedium,
				color = MaterialTheme.colorScheme.onBackground,
				fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
			)
		},
		confirmButton = {
			if (isLoggingOut) {
				Row(
					modifier = Modifier
						.fillMaxWidth(),
					horizontalArrangement = Arrangement.Center,
					verticalAlignment = Alignment.CenterVertically,
				) {
					Text(
						text = stringResource(R.string.logging_out),
						style = MaterialTheme.typography.bodyLarge,
						color = MaterialTheme.colorScheme.onPrimaryContainer,
					)
					Spacer(modifier = Modifier.width(12.dp))
					CircularProgressIndicator(
						color = LocalNymColors.current.warning,
						modifier = Modifier.size(24.dp),
						strokeWidth = 4.dp,
					)
				}
			} else {
				MainStyledButton(
					onClick = {
						onConfirm()
					},
					textColor = MaterialTheme.colorScheme.onPrimaryContainer,
					content = {
						Text(
							stringResource(R.string.log_out),
							style = MaterialTheme.typography.bodyLarge,
						)
					},
					modifier = Modifier
						.fillMaxWidth()
						.height(40.dp.scaledHeight()),
					color = LocalNymColors.current.buttonErrorText,
					borderStroke = BorderStroke(width = 1.dp, color = LocalNymColors.current.buttonErrorBorder),
				)
			}
		},
		dismissButton = {
			if (!isLoggingOut) {
				TransparentButton(
					onClick = onDismiss,
					content = {
						Text(
							stringResource(R.string.cancel),
							style = MaterialTheme.typography.bodyLarge,
							color = MaterialTheme.colorScheme.onPrimaryContainer,
						)
					},
					modifier = Modifier
						.fillMaxWidth()
						.height(40.dp.scaledHeight()),
				)
			}
		},
	)
}

@Composable
@Preview(uiMode = Configuration.UI_MODE_NIGHT_YES)
internal fun PreviewLogoutDialog() {
	NymVPNTheme(Theme.default()) {
		LogoutDialog(show = true, isLoggingOut = false, {}, {})
	}
}
