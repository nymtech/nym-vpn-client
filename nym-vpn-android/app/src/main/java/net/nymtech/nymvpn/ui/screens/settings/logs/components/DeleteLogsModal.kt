package net.nymtech.nymvpn.ui.screens.settings.logs.components

import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Delete
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.common.Modal
import net.nymtech.nymvpn.ui.common.buttons.MainStyledButton
import net.nymtech.nymvpn.ui.theme.CustomTypography
import net.nymtech.nymvpn.util.extensions.scaledHeight

@Composable
fun DeleteLogsModal(show: Boolean, onDismiss: () -> Unit, onConfirm: () -> Unit) {
	Modal(
		show = show,
		onDismiss = onDismiss,
		title = {
			Text(
				stringResource(R.string.delete_logs_title).uppercase(),
				style = CustomTypography.labelHuge,
				fontFamily = FontFamily(Font(R.font.lab_grotesque_mono)),
			)
		},
		text = {
			Text(
				stringResource(R.string.delete_logs_description),
				textAlign = TextAlign.Center,
				style = MaterialTheme.typography.bodyMedium,
			)
		},
		icon = Icons.Outlined.Delete,
		confirmButton = {
			MainStyledButton(
				onClick = onConfirm,
				content = { Text(stringResource(R.string.yes).uppercase(), fontFamily = FontFamily(Font(R.font.lab_grotesque_mono))) },
				modifier = Modifier.fillMaxWidth().height(40.dp.scaledHeight()),
			)
		},
	)
}
