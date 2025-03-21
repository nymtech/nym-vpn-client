package net.nymtech.nymvpn.ui.screens.settings.developer.components

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.res.vectorResource
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.AppUiState
import net.nymtech.nymvpn.ui.common.buttons.surface.SelectionItem
import net.nymtech.nymvpn.ui.screens.settings.components.SettingsGroup
import net.nymtech.nymvpn.ui.theme.iconSize
import net.nymtech.nymvpn.util.extensions.scaledHeight
import net.nymtech.nymvpn.util.extensions.scaledWidth

@Composable
fun MixnetStateSection(appUiState: AppUiState) {
	appUiState.managerState.mixnetConnectionState?.let { mixnetState ->
		SettingsGroup(
			items = listOf(
				SelectionItem(
					title = {
						Row(
							verticalAlignment = Alignment.CenterVertically,
							modifier = Modifier.fillMaxWidth().padding(vertical = 4.dp.scaledHeight()),
						) {
							Row(
								verticalAlignment = Alignment.CenterVertically,
								modifier = Modifier
									.weight(4f, false)
									.fillMaxWidth(),
							) {
								Icon(
									imageVector = ImageVector.vectorResource(R.drawable.mixnet),
									contentDescription = "Mixnet",
									modifier = Modifier.size(iconSize),
								)
								Column(
									horizontalAlignment = Alignment.Start,
									verticalArrangement = Arrangement.spacedBy(2.dp, Alignment.CenterVertically),
									modifier = Modifier
										.fillMaxWidth()
										.padding(start = 16.dp.scaledWidth())
										.padding(vertical = 6.dp.scaledHeight()),
								) {
									Text(
										"Mixnet client state",
										style = MaterialTheme.typography.bodyMedium.copy(MaterialTheme.colorScheme.onSurface),
									)
								}
							}
						}
					},
					description = {
						Column {
							Text(
								"Ipv4: ${mixnetState.ipv4State}",
								style = MaterialTheme.typography.bodyMedium.copy(MaterialTheme.colorScheme.outline),
							)
							Text(
								"Ipv6: ${mixnetState.ipv6State}",
								style = MaterialTheme.typography.bodyMedium.copy(MaterialTheme.colorScheme.outline),
							)
						}
					},
					trailing = null,
				),
			),
		)
	}
}
