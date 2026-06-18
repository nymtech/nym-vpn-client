package net.nymtech.nymvpn.ui.common

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Info
import androidx.compose.material3.AlertDialogDefaults
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import androidx.compose.ui.window.Dialog
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.common.buttons.MainStyledButton
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.util.extensions.scaledHeight

@Composable
fun ModalContent(
	title: @Composable () -> Unit,
	text: @Composable () -> Unit,
	icon: ImageVector = Icons.Outlined.Info,
	iconTint: Color? = null,
	iconSize: Dp? = null,
	description: String = stringResource(R.string.info),
	confirmButton: @Composable () -> Unit = {},
	dismissButton: @Composable () -> Unit = {},
) {
	Surface(
		shape = AlertDialogDefaults.shape,
		color = MaterialTheme.colorScheme.surface,
		tonalElevation = 0.dp,
	) {
		Column(
			modifier = Modifier.padding(24.dp),
			horizontalAlignment = Alignment.CenterHorizontally,
			verticalArrangement = Arrangement.spacedBy(16.dp),
		) {
			Icon(
				icon,
				description,
				tint = iconTint ?: MaterialTheme.colorScheme.onBackground,
				modifier = Modifier.then(
					if (iconSize != null) Modifier.size(iconSize) else Modifier,
				),
			)
			title()
			text()
			confirmButton()
			dismissButton()
		}
	}
}

@Composable
fun Modal(
	show: Boolean,
	onDismiss: () -> Unit,
	title: @Composable () -> Unit,
	text: @Composable () -> Unit,
	icon: ImageVector = Icons.Outlined.Info,
	iconTint: Color? = null,
	iconSize: Dp? = null,
	description: String = stringResource(R.string.info),
	confirmButton: @Composable () -> Unit = {
		MainStyledButton(
			onClick = onDismiss,
			content = {
				Text(text = stringResource(id = R.string.okay), fontFamily = FontFamily(Font(R.font.lab_grotesque_mono)))
			},
			modifier = Modifier
				.fillMaxWidth()
				.height(40.dp.scaledHeight()),
		)
	},
	dismissButton: @Composable () -> Unit = {},
) {
	if (show) {
		Dialog(onDismissRequest = onDismiss) {
			ModalContent(
				title = title,
				text = text,
				icon = icon,
				iconTint = iconTint,
				iconSize = iconSize,
				description = description,
				confirmButton = confirmButton,
				dismissButton = dismissButton,
			)
		}
	}
}

@Preview
@Composable
private fun ModalContentPreview() {
	NymVPNTheme(Theme.default()) {
		ModalContent(
			title = { Text("Dialog Title") },
			text = { Text("This is the dialog body text.") },
			confirmButton = {
				MainStyledButton(
					onClick = {},
					content = {
						Text(text = "OK", fontFamily = FontFamily(Font(R.font.lab_grotesque_mono)))
					},
					modifier = Modifier
						.fillMaxWidth()
						.height(40.dp),
				)
			},
		)
	}
}
