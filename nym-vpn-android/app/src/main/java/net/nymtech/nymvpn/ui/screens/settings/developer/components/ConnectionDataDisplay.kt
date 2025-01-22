package net.nymtech.nymvpn.ui.screens.settings.developer.components

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalClipboardManager
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.util.extensions.scaledWidth
import nym_vpn_lib.ConnectionData
import nym_vpn_lib.TunnelConnectionData

@Composable
fun ConnectionDataDisplay(connectionData: ConnectionData) {
	val clipboard = LocalClipboardManager.current
	connectionData.let {
		Column(modifier = Modifier.padding(end = 10.dp.scaledWidth())) {
			Text(
				"Entry gatewayId: ${it.entryGateway.id}",
				style = MaterialTheme.typography.bodyMedium.copy(MaterialTheme.colorScheme.outline),
				modifier = Modifier.clickable { clipboard.setText(AnnotatedString(it.entryGateway.id)) },
			)
			Text(
				"Exit gatewayId: ${it.exitGateway.id}",
				style = MaterialTheme.typography.bodyMedium.copy(MaterialTheme.colorScheme.outline),
				modifier = Modifier.clickable { clipboard.setText(AnnotatedString(it.exitGateway.id)) },
			)
			it.connectedAt?.let { connectedAt ->
				Text(
					"Connected at: $connectedAt",
					style = MaterialTheme.typography.bodyMedium.copy(MaterialTheme.colorScheme.outline),
					modifier = Modifier.clickable { clipboard.setText(AnnotatedString(connectedAt.toString())) },
				)
			}
			when (val details = it.tunnel) {
				is TunnelConnectionData.Mixnet -> {
					Text(
						"Ipv4: ${details.v1.ipv4}",
						style = MaterialTheme.typography.bodyMedium.copy(MaterialTheme.colorScheme.outline),
						modifier = Modifier.clickable { clipboard.setText(AnnotatedString(details.v1.ipv4)) },
					)
					Text(
						"Ipv6: ${details.v1.ipv6}",
						style = MaterialTheme.typography.bodyMedium.copy(MaterialTheme.colorScheme.outline),
						modifier = Modifier.clickable { clipboard.setText(AnnotatedString(details.v1.ipv6)) },
					)
					Text(
						"Exit IPR: ${details.v1.exitIpr}",
						style = MaterialTheme.typography.bodyMedium.copy(MaterialTheme.colorScheme.outline),
						modifier = Modifier.clickable { clipboard.setText(AnnotatedString(details.v1.exitIpr.nymAddress)) },
					)
					Text(
						"Nym address: ${details.v1.nymAddress}",
						style = MaterialTheme.typography.bodyMedium.copy(MaterialTheme.colorScheme.outline),
						modifier = Modifier.clickable { clipboard.setText(AnnotatedString(details.v1.nymAddress.nymAddress)) },
					)
				}
				is TunnelConnectionData.Wireguard -> {
					Text(
						"Entry endpoint: ${details.v1.entry.endpoint}",
						style = MaterialTheme.typography.bodyMedium.copy(MaterialTheme.colorScheme.outline),
						modifier = Modifier.clickable { clipboard.setText(AnnotatedString(details.v1.entry.endpoint)) },
					)
					Text(
						"Entry pub key: ${details.v1.entry.publicKey}",
						style = MaterialTheme.typography.bodyMedium.copy(MaterialTheme.colorScheme.outline),
						modifier = Modifier.clickable { clipboard.setText(AnnotatedString(details.v1.entry.publicKey)) },

					)
					Text(
						"Entry Ipv4: ${details.v1.entry.privateIpv4}",
						style = MaterialTheme.typography.bodyMedium.copy(MaterialTheme.colorScheme.outline),
						modifier = Modifier.clickable { clipboard.setText(AnnotatedString(details.v1.entry.privateIpv4)) },

					)
					Text(
						"Exit endpoint: ${details.v1.exit.endpoint}",
						style = MaterialTheme.typography.bodyMedium.copy(MaterialTheme.colorScheme.outline),
						modifier = Modifier.clickable { clipboard.setText(AnnotatedString(details.v1.exit.endpoint)) },

					)
					Text(
						"Exit pub key: ${details.v1.exit.publicKey}",
						style = MaterialTheme.typography.bodyMedium.copy(MaterialTheme.colorScheme.outline),
						modifier = Modifier.clickable { clipboard.setText(AnnotatedString(details.v1.exit.publicKey)) },

					)
					Text(
						"Exit Ipv4: ${details.v1.exit.privateIpv4}",
						style = MaterialTheme.typography.bodyMedium.copy(MaterialTheme.colorScheme.outline),
						modifier = Modifier.clickable { clipboard.setText(AnnotatedString(details.v1.exit.privateIpv4)) },

					)
				}
			}
		}
	}
}
