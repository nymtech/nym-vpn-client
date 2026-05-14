package net.nymtech.nymvpn.ui.screens.settings.modal

import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Dns
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.common.Modal
import net.nymtech.nymvpn.ui.common.buttons.MainStyledButton
import net.nymtech.nymvpn.ui.theme.CustomTypography
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.util.extensions.scaledHeight

@Composable
fun PrivateDnsDialog(showPrivateDnsDialog: Boolean, onClickSettings: () -> Unit, onDismiss: () -> Unit) {
	Modal(
		show = showPrivateDnsDialog,
		icon = Icons.Outlined.Dns,
		onDismiss = onDismiss,
		title = {
			Text(
				text = stringResource(R.string.private_dns_title),
				style = CustomTypography.labelHuge,
				color = MaterialTheme.colorScheme.onPrimaryContainer,
				fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
			)
		},
		text = {},
		confirmButton = {
			MainStyledButton(
				onClick = onClickSettings,
				textColor = MaterialTheme.colorScheme.onPrimary,
				content = {
					Text(
						stringResource(R.string.private_dns_button),
						style = MaterialTheme.typography.titleMedium,
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
		PrivateDnsDialog(
			true,
			onClickSettings = {},
			onDismiss = {},
		)
	}
}
