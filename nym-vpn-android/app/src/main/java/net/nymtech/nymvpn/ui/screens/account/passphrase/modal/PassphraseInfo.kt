package net.nymtech.nymvpn.ui.screens.account.passphrase.modal

import android.content.res.Configuration
import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.Login
import androidx.compose.material.icons.filled.Mail
import androidx.compose.material.icons.rounded.Password
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.res.vectorResource
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.common.buttons.TransparentButton
import net.nymtech.nymvpn.ui.screens.account.passphrase.components.TripleRow
import net.nymtech.nymvpn.ui.theme.CustomTypography
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.util.extensions.scaledHeight

@Composable
fun PassphraseInfo(show: Boolean, onDismiss: () -> Unit) {
	if (show) {
		AlertDialog(
			containerColor = MaterialTheme.colorScheme.primaryContainer,
			onDismissRequest = { onDismiss() },
			tonalElevation = 0.dp,
			confirmButton = {
				TransparentButton(
					onClick = {
						onDismiss()
					},
					content = {
						Text(
							stringResource(R.string.button_done),
							fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
							color = MaterialTheme.colorScheme.onPrimaryContainer,
						)
					},
					modifier = Modifier.fillMaxWidth().height(40.dp.scaledHeight()),
					borderStroke = BorderStroke(width = 1.dp, color = MaterialTheme.colorScheme.onPrimaryContainer),
				)
			},
			text = {
				Column(
					horizontalAlignment = Alignment.CenterHorizontally,
					verticalArrangement = Arrangement.spacedBy(16.dp, Alignment.Top),
				) {
					Text(
						text = stringResource(R.string.passphrase_info_title),
						style = CustomTypography.titleMediumPlus,
						color = MaterialTheme.colorScheme.onPrimaryContainer,
						fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
					)
					Text(
						text = stringResource(R.string.passphrase_info_description),
						style = MaterialTheme.typography.bodyMedium,
						color = MaterialTheme.colorScheme.onPrimaryContainer,
						textAlign = TextAlign.Center,
						fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
					)
					Text(
						text = stringResource(R.string.passphrase_info_privacy_removed),
						style = MaterialTheme.typography.labelLarge,
						color = MaterialTheme.colorScheme.onPrimaryContainer,
						textAlign = TextAlign.Center,
						fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
					)
					TripleRow(
						items = listOf(
							InfoItem(
								label = stringResource(R.string.passphrase_info_stored_accounts),
								icon = ImageVector.vectorResource(id = R.drawable.ic_database),
								negative = true,
							),
							InfoItem(
								label = stringResource(R.string.passphrase_info_personal_logins),
								icon = Icons.Filled.Mail,
								negative = true,
							),
							InfoItem(
								label = stringResource(R.string.passphrase_info_access_resets),
								icon = Icons.Rounded.Password,
								negative = true,
							),
						),
					)
					Text(
						text = stringResource(R.string.passphrase_info_use),
						style = MaterialTheme.typography.labelLarge,
						color = MaterialTheme.colorScheme.onPrimaryContainer,
						textAlign = TextAlign.Center,
						fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
					)
					TripleRow(
						items = listOf(
							InfoItem(
								label = stringResource(R.string.passphrase_info_secure_login),
								icon = ImageVector.vectorResource(id = R.drawable.ic_encrypted),
								negative = false,
							),
							InfoItem(
								label = stringResource(R.string.passphrase_info_keeps_anonymous),
								icon = ImageVector.vectorResource(id = R.drawable.ic_mask),
								negative = false,
							),
							InfoItem(
								label = stringResource(R.string.passphrase_info_belongs),
								icon = Icons.AutoMirrored.Filled.Login,
								negative = false,
							),
						),
					)
				}
			},

		)
	}
}

@Composable
@Preview(uiMode = Configuration.UI_MODE_NIGHT_YES)
internal fun PreviewPassphraseScreen() {
	NymVPNTheme(Theme.default()) {
		PassphraseInfo(true) { }
	}
}
