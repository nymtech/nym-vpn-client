package net.nymtech.nymvpn.ui.screens.main.modal

import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
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
fun BatteryModal(showBatteryDialog: Boolean, onClickSettings: () -> Unit, onDismiss: () -> Unit) {
	Modal(
		show = showBatteryDialog,
		onDismiss = onDismiss,
		title = {
			Text(
				text = stringResource(R.string.battery_opt_title),
				style = CustomTypography.labelHuge,
				color = MaterialTheme.colorScheme.onPrimaryContainer,
				fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
			)
		},
		text = {
			Text(
				stringResource(R.string.battery_opt_descr),
				textAlign = TextAlign.Center,
				style = MaterialTheme.typography.bodyMedium,
				color = MaterialTheme.colorScheme.onPrimaryContainer,
				fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
			)
		},
		confirmButton = {
			MainStyledButton(
				onClick = onClickSettings,
				textColor = Color.Black,
				content = {
					Text(
						stringResource(R.string.settings),
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
				onClick = onDismiss,
				content = {
					Text(
						stringResource(R.string.skip),
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

@Preview
@Composable
private fun BatteryModalPreview() {
	NymVPNTheme(Theme.default()) {
		BatteryModal(true, {}, {})
	}
}
