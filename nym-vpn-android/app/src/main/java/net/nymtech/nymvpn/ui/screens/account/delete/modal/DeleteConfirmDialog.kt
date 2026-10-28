package net.nymtech.nymvpn.ui.screens.account.delete.modal

import android.content.res.Configuration
import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Warning
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
import net.nymtech.nymvpn.ui.theme.LocalNymColors
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.util.extensions.scaledHeight

@Composable
fun DeleteConfirmDialog(show: Boolean, onConfirm: () -> Unit, onKeep: () -> Unit) {
	Modal(
		show = show,
		onDismiss = onKeep,
		icon = Icons.Outlined.Warning,
		iconTint = LocalNymColors.current.warning,
		title = {
			Text(
				text = stringResource(R.string.delete_account_dialog_title),
				style = MaterialTheme.typography.titleLarge,
				color = MaterialTheme.colorScheme.onPrimaryContainer,
			)
		},
		text = {
			Text(
				stringResource(R.string.delete_account_dialog_description),
				textAlign = TextAlign.Center,
				style = MaterialTheme.typography.bodyMedium,
				color = MaterialTheme.colorScheme.onBackground,
				fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
			)
		},
		confirmButton = {
			MainStyledButton(
				onClick = onConfirm,
				textColor = MaterialTheme.colorScheme.onPrimaryContainer,
				content = {
					Text(
						stringResource(R.string.account_info_delete_button),
						style = MaterialTheme.typography.bodyLarge,
					)
				},
				modifier = Modifier
					.fillMaxWidth()
					.height(40.dp.scaledHeight()),
				color = LocalNymColors.current.buttonErrorText,
				borderStroke = BorderStroke(width = 1.dp, color = LocalNymColors.current.buttonErrorBorder),
			)
		},
		dismissButton = {
			TransparentButton(
				onClick = onKeep,
				content = {
					Text(
						stringResource(R.string.delete_account_keep_button),
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
internal fun PreviewDeleteConfirmDialog() {
	NymVPNTheme(Theme.default()) {
		DeleteConfirmDialog(show = true, onConfirm = {}, onKeep = {})
	}
}
