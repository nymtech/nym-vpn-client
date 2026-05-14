package net.nymtech.nymvpn.ui.screens.settings.dns.modal

import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Settings
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
import net.nymtech.nymvpn.ui.theme.CustomTypography
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.util.extensions.scaledHeight

@Composable
fun SaveChangesModal(showSaveChangesDialog: Boolean, confirmTextResId: Int, onClickSave: () -> Unit, onDiscard: () -> Unit, onDismiss: () -> Unit) {
	Modal(
		show = showSaveChangesDialog,
		icon = Icons.Outlined.Settings,
		onDismiss = onDismiss,
		title = {
			Text(
				text = stringResource(R.string.dns_save_dialog_title),
				style = CustomTypography.labelHuge,
				color = MaterialTheme.colorScheme.onPrimaryContainer,
				fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
			)
		},
		text = {
			Text(
				stringResource(R.string.dns_save_dialog_description),
				textAlign = TextAlign.Center,
				modifier = Modifier.fillMaxWidth(),
				style = MaterialTheme.typography.bodyMedium,
				color = MaterialTheme.colorScheme.onPrimaryContainer,
				fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
			)
		},
		confirmButton = {
			MainStyledButton(
				onClick = onClickSave,
				textColor = MaterialTheme.colorScheme.onPrimary,
				content = {
					Text(
						stringResource(confirmTextResId),
						fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
					)
				},
				modifier = Modifier
					.fillMaxWidth()
					.height(40.dp.scaledHeight()),
			)
		},
		dismissButton = {
			TransparentButton(
				onClick = onDiscard,
				content = {
					Text(
						stringResource(R.string.button_discard),
						fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
						color = MaterialTheme.colorScheme.error,
					)
				},
				modifier = Modifier
					.fillMaxWidth()
					.height(40.dp.scaledHeight()),
			)
		},
	)
}

@Preview
@Composable
private fun SaveChangesModalPreview() {
	NymVPNTheme(Theme.default()) {
		SaveChangesModal(
			showSaveChangesDialog = true,
			confirmTextResId = R.string.dns_custom_button_save,
			onClickSave = {},
			onDiscard = {},
			onDismiss = {},
		)
	}
}
