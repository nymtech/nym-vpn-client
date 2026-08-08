package net.nymtech.nymvpn.ui.screens.settings.tunneling.components

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Shield
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.res.vectorResource
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.tooling.preview.PreviewLightDark
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.common.Modal
import net.nymtech.nymvpn.ui.common.buttons.MainStyledButton
import net.nymtech.nymvpn.ui.theme.CustomTypography
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.util.extensions.scaledHeight

@Composable
internal fun SplitTunnelingInfoModal(showModal: Boolean, onDismiss: () -> Unit) {
	Modal(
		show = showModal,
		onDismiss = onDismiss,
		title = {
			Text(
				text = stringResource(R.string.split_tunnel_info_modal_title),
				style = CustomTypography.titleMediumPlus,
				color = MaterialTheme.colorScheme.onPrimaryContainer,
			)
		},
		text = {
			Column {
				Text(
					stringResource(R.string.split_tunnel_info_modal_desc),
					style = MaterialTheme.typography.bodyMedium,
					color = MaterialTheme.colorScheme.onPrimaryContainer,
				)
				Row(
					modifier = Modifier.fillMaxWidth().padding(top = 16.dp),
					horizontalArrangement = Arrangement.spacedBy(4.dp),
					verticalAlignment = Alignment.CenterVertically,
				) {
					Icon(
						ImageVector.vectorResource(R.drawable.split),
						stringResource(R.string.split_tunneling_direct),
						Modifier
							.size(20.dp)
							.align(Alignment.CenterVertically),
						tint = MaterialTheme.colorScheme.onBackground,
					)
					Text(
						text = stringResource(id = R.string.split_tunneling_direct),
						style = MaterialTheme.typography.bodyMedium,
						color = MaterialTheme.colorScheme.onPrimaryContainer,
						fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
						fontWeight = FontWeight(700),
					)
				}
				Text(
					text = stringResource(R.string.split_tunnel_info_direct_desc),
					modifier = Modifier.padding(top = 8.dp),
					style = MaterialTheme.typography.bodyMedium,
					color = MaterialTheme.colorScheme.onPrimaryContainer,
				)

				Row(
					modifier = Modifier
						.fillMaxWidth()
						.padding(top = 16.dp),
					horizontalArrangement = Arrangement.spacedBy(4.dp),
					verticalAlignment = Alignment.CenterVertically,
				) {
					Icon(
						Icons.Filled.Shield,
						stringResource(R.string.split_tunneling_via_vpn),
						Modifier
							.size(20.dp)
							.align(Alignment.CenterVertically),
						tint = MaterialTheme.colorScheme.onBackground,
					)
					Text(
						stringResource(id = R.string.split_tunneling_via_vpn),
						style = MaterialTheme.typography.bodyMedium,
						color = MaterialTheme.colorScheme.onPrimaryContainer,
						fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
						fontWeight = FontWeight(700),
					)
				}
				Text(
					text = stringResource(R.string.split_tunneling_via_desc),
					modifier = Modifier.padding(top = 8.dp),
					style = MaterialTheme.typography.bodyMedium,
					color = MaterialTheme.colorScheme.onPrimaryContainer,
				)
				Text(
					text = stringResource(R.string.split_tunnel_info_lockdown_privacy_note),
					modifier = Modifier.padding(top = 16.dp),
					style = MaterialTheme.typography.bodyMedium,
					color = MaterialTheme.colorScheme.onPrimaryContainer,
				)
			}
		},
		confirmButton = {
			MainStyledButton(
				onClick = onDismiss,
				textColor = Color.Black,
				content = { Text(stringResource(R.string.ok), fontFamily = FontFamily(Font(R.font.lab_grotesque_regular))) },
				modifier = Modifier
					.fillMaxWidth()
					.height(40.dp.scaledHeight()),
			)
		},
	)
}

@Composable
@PreviewLightDark
private fun ExitServerDetailsModalPreview() {
	NymVPNTheme(Theme.default()) {
		SplitTunnelingInfoModal(true, onDismiss = {})
	}
}
