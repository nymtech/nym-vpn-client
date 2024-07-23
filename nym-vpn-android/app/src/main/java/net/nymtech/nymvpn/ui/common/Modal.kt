package net.nymtech.nymvpn.ui.common

import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Info
import androidx.compose.material3.AlertDialog
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.common.buttons.MainStyledButton

@Composable
fun Modal(show: Boolean, onDismiss: () -> Unit, title: @Composable () -> Unit, text: @Composable () -> Unit) {
	if (show) {
		AlertDialog(
			containerColor = MaterialTheme.colorScheme.surfaceContainer,
			onDismissRequest = { onDismiss() },
			tonalElevation = 0.dp,
			confirmButton = {
				MainStyledButton(
					onClick = {
						onDismiss()
					},
					content = {
						Text(text = stringResource(id = R.string.okay))
					},
				)
			},
			icon = {
				val icon = Icons.Outlined.Info
				Icon(icon, icon.name, tint = MaterialTheme.colorScheme.onSurface)
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
