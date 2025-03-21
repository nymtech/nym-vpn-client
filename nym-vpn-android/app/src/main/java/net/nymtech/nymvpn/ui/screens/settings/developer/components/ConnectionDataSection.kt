package net.nymtech.nymvpn.ui.screens.settings.developer.components

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Bolt
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.ui.AppUiState
import net.nymtech.nymvpn.ui.common.buttons.surface.SelectionItem
import net.nymtech.nymvpn.ui.screens.settings.components.SettingsGroup
import net.nymtech.nymvpn.ui.theme.iconSize
import net.nymtech.nymvpn.util.extensions.scaledHeight
import net.nymtech.nymvpn.util.extensions.scaledWidth

@Composable
fun ConnectionDataSection(appUiState: AppUiState) {
	appUiState.managerState.connectionData?.let { connectionData ->
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
									Icons.Outlined.Bolt,
									"Tunnel details",
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
										"Tunnel details",
										style = MaterialTheme.typography.bodyMedium.copy(MaterialTheme.colorScheme.onSurface),
									)
								}
							}
						}
					},
					description = { ConnectionDataDisplay(connectionData) },
					trailing = null,
				),
			),
		)
	}
}
