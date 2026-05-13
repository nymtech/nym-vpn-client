package net.nymtech.nymvpn.ui.screens.main.modal

import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Info
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
fun NetworkStatsModal(showNetworkStatsDialog: Boolean, onDismiss: () -> Unit, onConfirm: () -> Unit) {
	Modal(
		show = showNetworkStatsDialog,
		onDismiss = onDismiss,
		title = {
			Text(
				stringResource(R.string.modal_network_stats_title),
				style = CustomTypography.labelHuge,
				color = MaterialTheme.colorScheme.onPrimaryContainer,
				fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
			)
		},
		text = {
			Text(
				stringResource(R.string.modal_network_stats_descr),
				textAlign = TextAlign.Center,
				style = MaterialTheme.typography.bodyMedium,
				color = MaterialTheme.colorScheme.onPrimaryContainer,
				fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
			)
		},
		icon = Icons.Outlined.Info,
		confirmButton = {
			MainStyledButton(
				onClick = onConfirm,
				textColor = Color.Black,
				content = {
					Text(
						stringResource(R.string.modal_network_stats_enable_button),
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
						stringResource(R.string.modal_network_stats_not_now_button),
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
private fun NetworkStatsModalPreview() {
	NymVPNTheme(Theme.default()) {
		NetworkStatsModal(true, {}, {})
	}
}
