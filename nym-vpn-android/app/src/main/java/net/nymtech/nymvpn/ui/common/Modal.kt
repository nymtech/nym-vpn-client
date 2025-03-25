package net.nymtech.nymvpn.ui.common

import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Info
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.common.buttons.MainStyledButton
import net.nymtech.nymvpn.util.extensions.scaledHeight

@Composable
fun Modal(
	show: Boolean,
	onDismiss: () -> Unit,
	title: @Composable () -> Unit,
	text: @Composable () -> Unit,
	icon: ImageVector = Icons.Outlined.Info,
	description: String = stringResource(R.string.info),
	confirmButton: @Composable () -> Unit = {
		MainStyledButton(
			onClick = {
				onDismiss()
			},
			content = {
				Text(text = stringResource(id = R.string.okay).uppercase(), fontFamily = FontFamily(Font(R.font.lab_grotesque_mono)))
			},
			modifier = Modifier.fillMaxWidth().height(40.dp.scaledHeight()),
		)
	},
	dismissButton: @Composable () -> Unit = {},
) {
	if (show) {
		AlertDialog(
			containerColor = MaterialTheme.colorScheme.surfaceContainer,
			onDismissRequest = { onDismiss() },
			tonalElevation = 0.dp,
			dismissButton = dismissButton,
			confirmButton = {
				confirmButton()
			},
			icon = {
				Icon(icon, description, tint = MaterialTheme.colorScheme.onSurface)
			},
			title = {
				title()
			},
			text = {
				text()
			},
		)
	}
}
