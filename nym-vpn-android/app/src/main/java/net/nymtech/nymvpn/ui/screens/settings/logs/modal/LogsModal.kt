package net.nymtech.nymvpn.ui.screens.settings.logs.modal

import android.content.res.Configuration
import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Delete
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.ImageVector
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
import net.nymtech.nymvpn.ui.theme.LocalNymColors
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.util.extensions.scaledHeight

@Composable
fun LogsModal(show: Boolean, onDismiss: () -> Unit, onConfirm: () -> Unit, title: String, description: String, buttonText: String, icon: ImageVector) {
	Modal(
		show = show,
		onDismiss = onDismiss,
		icon = icon,
		title = {
			Text(
				text = title,
				style = CustomTypography.labelHuge,
				color = MaterialTheme.colorScheme.onPrimaryContainer,
			)
		},
		text = {
			Text(
				description,
				textAlign = TextAlign.Center,
				style = MaterialTheme.typography.bodyMedium,
				color = MaterialTheme.colorScheme.onPrimaryContainer,
			)
		},
		confirmButton = {
			MainStyledButton(
				onClick = {
					onConfirm()
				},
				textColor = MaterialTheme.colorScheme.onPrimaryContainer,
				content = {
					Text(
						buttonText,
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
				onClick = onDismiss,
				content = {
					Text(
						stringResource(R.string.cancel),
						fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
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
internal fun PreviewLogsModal() {
	NymVPNTheme(Theme.default()) {
		LogsModal(
			show = true,
			{},
			{},
			title = stringResource(R.string.logs_delete_title),
			description = stringResource(R.string.logs_delete_description),
			buttonText = stringResource(R.string.logs_delete_title),
			icon = Icons.Outlined.Delete,
		)
	}
}
