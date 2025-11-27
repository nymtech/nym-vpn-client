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
import net.nymtech.nymvpn.ui.theme.CustomColors
import net.nymtech.nymvpn.ui.theme.CustomTypography
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.util.extensions.scaledHeight

@Composable
fun LogoutDialog(show: Boolean, isLoggingOut: Boolean, onDismiss: () -> Unit, onConfirm: () -> Unit) {
	Modal(
		show = show,
		onDismiss = onDismiss,
		icon = Icons.AutoMirrored.Default.Logout,
		title = {
			Text(
				text = stringResource(R.string.log_out_title),
				style = CustomTypography.labelHuge,
				color = MaterialTheme.colorScheme.onBackground,
				fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
			)
		},
		text = {
			Text(
				stringResource(R.string.log_out_body),
				textAlign = TextAlign.Center,
				style = MaterialTheme.typography.bodyMedium,
				color = MaterialTheme.colorScheme.outline,
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
						style = MaterialTheme.typography.labelLarge,
						fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
						color = MaterialTheme.colorScheme.onBackground,
					)
					Spacer(modifier = Modifier.width(12.dp))
					CircularProgressIndicator(
						color = CustomColors.warning,
						modifier = Modifier.size(24.dp),
						strokeWidth = 2.dp,
					)
				}
			} else {
				MainStyledButton(
					onClick = {
						onConfirm()
					},
					content = {
						Text(
							stringResource(R.string.log_out),
							fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
							color = MaterialTheme.colorScheme.onSurface,
						)
					},
					modifier = Modifier
						.fillMaxWidth()
						.height(40.dp.scaledHeight()),
					color = CustomColors.buttonRedTransparent,
					borderStroke = BorderStroke(width = 1.dp, color = CustomColors.buttonRedTransparentBorder),
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
							fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
							color = MaterialTheme.colorScheme.onSurface,
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
		LogoutDialog(show = true, isLoggingOut = true, {}, {})
	}
}
