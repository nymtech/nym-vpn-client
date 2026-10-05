package net.nymtech.nymvpn.ui.common.modal

import android.content.res.Configuration
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Lock
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
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
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.util.extensions.scaledHeight

@Composable
fun ScreenLockSetupDialog(show: Boolean, body: String, onSetUpClick: () -> Unit, onDismiss: () -> Unit) {
	Modal(
		show = show,
		onDismiss = onDismiss,
		icon = Icons.Outlined.Lock,
		description = stringResource(R.string.screen_lock_setup_title),
		title = {
			Text(
				text = stringResource(R.string.screen_lock_setup_title),
				style = MaterialTheme.typography.titleLarge,
				color = MaterialTheme.colorScheme.onPrimaryContainer,
				textAlign = TextAlign.Center,
			)
		},
		text = {
			Text(
				text = body,
				textAlign = TextAlign.Center,
				style = MaterialTheme.typography.bodyMedium,
				color = MaterialTheme.colorScheme.onBackground,
				fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
			)
		},
		confirmButton = {
			MainStyledButton(
				onClick = onSetUpClick,
				textColor = MaterialTheme.colorScheme.onPrimary,
				content = {
					Text(
						stringResource(R.string.screen_lock_setup_button),
						style = MaterialTheme.typography.bodyLarge,
					)
				},
				modifier = Modifier
					.fillMaxWidth()
					.height(40.dp.scaledHeight()),
			)
		},
		dismissButton = {
			TransparentButton(
				onClick = onDismiss,
				content = {
					Text(
						stringResource(R.string.screen_lock_setup_not_now),
						style = MaterialTheme.typography.bodyLarge,
						color = MaterialTheme.colorScheme.onPrimaryContainer,
					)
				},
				modifier = Modifier
					.fillMaxWidth()
					.height(40.dp.scaledHeight()),
			)
		},
	)
}

@Composable
@Preview(uiMode = Configuration.UI_MODE_NIGHT_YES)
internal fun PreviewScreenLockSetupDialog() {
	NymVPNTheme(Theme.default()) {
		ScreenLockSetupDialog(
			show = true,
			body = stringResource(R.string.screen_lock_setup_passphrase_body),
			onSetUpClick = {},
			onDismiss = {},
		)
	}
}
