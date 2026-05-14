package net.nymtech.nymvpn.ui.screens.settings.developer.components

import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Key
import androidx.compose.material.icons.outlined.Place
import androidx.compose.material3.DropdownMenuItem
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.ExposedDropdownMenuBox
import androidx.compose.material3.ExposedDropdownMenuDefaults
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.MenuAnchorType
import androidx.compose.material3.Text
import androidx.compose.material3.TextField
import androidx.compose.runtime.Composable
import androidx.compose.runtime.derivedStateOf
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.ui.Modifier
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.ui.AppUiState
import net.nymtech.nymvpn.ui.common.buttons.surface.SelectionItem
import net.nymtech.nymvpn.ui.screens.settings.components.SettingsGroup
import net.nymtech.nymvpn.ui.screens.settings.developer.CredentialMode
import net.nymtech.nymvpn.ui.theme.iconSize
import net.nymtech.nymvpn.util.extensions.scaledWidth
import net.nymtech.vpn.backend.Tunnel

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun DeveloperOptionsSection(
	appUiState: AppUiState,
	environmentExpanded: Boolean,
	onEnvironmentExpandedChange: (Boolean) -> Unit,
	credentialExpanded: Boolean,
	onCredentialExpandedChange: (Boolean) -> Unit,
	onLogout: suspend () -> Unit,
	onEnvironmentChange: suspend (Tunnel.Environment) -> Unit,
	onCredentialOverride: (Boolean?) -> Unit,
) {
	val credentialMode by remember {
		derivedStateOf {
			when (appUiState.settings.isCredentialMode) {
				true -> CredentialMode.ON
				false -> CredentialMode.OFF
				null -> CredentialMode.DEFAULT
			}
		}
	}
	val scope = rememberCoroutineScope()

	SettingsGroup(
		items = listOf(
			SelectionItem(
				leading = {
					Icon(
						Icons.Outlined.Place,
						"Location",
						modifier = Modifier.size(iconSize.scaledWidth()),
					)
				},
				title = {
					ExposedDropdownMenuBox(
						expanded = environmentExpanded,
						onExpandedChange = onEnvironmentExpandedChange,
					) {
						TextField(
							value = appUiState.vpnConfig.network.networkName().uppercase(),
							onValueChange = {},
							readOnly = true,
							textStyle = MaterialTheme.typography.bodyMedium.copy(MaterialTheme.colorScheme.onPrimaryContainer),
							trailingIcon = { ExposedDropdownMenuDefaults.TrailingIcon(expanded = environmentExpanded) },
							modifier = Modifier.menuAnchor(MenuAnchorType.PrimaryNotEditable, true),
						)
						ExposedDropdownMenu(
							expanded = environmentExpanded,
							onDismissRequest = { onEnvironmentExpandedChange(false) },
						) {
							enumValues<Tunnel.Environment>().forEach { item ->
								DropdownMenuItem(
									text = { Text(item.name) },
									onClick = {
										scope.launch {
											if (appUiState.vpnConfig.network == item) return@launch
											onLogout()
											onEnvironmentChange(item)
											onEnvironmentExpandedChange(false)
										}
									},
								)
							}
						}
					}
				},
				description = {
					Text(
						"Environment",
						style = MaterialTheme.typography.bodyMedium.copy(MaterialTheme.colorScheme.onBackground),
					)
				},
				trailing = null,
			),
			SelectionItem(
				leading = {
					Icon(
						Icons.Outlined.Key,
						"Key",
						modifier = Modifier.size(iconSize.scaledWidth()),
					)
				},
				title = {
					ExposedDropdownMenuBox(
						expanded = credentialExpanded,
						onExpandedChange = onCredentialExpandedChange,
					) {
						TextField(
							value = credentialMode.name,
							onValueChange = {},
							readOnly = true,
							textStyle = MaterialTheme.typography.bodyMedium.copy(MaterialTheme.colorScheme.onPrimaryContainer),
							trailingIcon = { ExposedDropdownMenuDefaults.TrailingIcon(expanded = credentialExpanded) },
							modifier = Modifier.menuAnchor(MenuAnchorType.PrimaryNotEditable, true),
						)
						ExposedDropdownMenu(
							expanded = credentialExpanded,
							onDismissRequest = { onCredentialExpandedChange(false) },
						) {
							enumValues<CredentialMode>().forEach { item ->
								DropdownMenuItem(
									text = { Text(item.name) },
									onClick = {
										if (credentialMode == item) return@DropdownMenuItem
										when (item) {
											CredentialMode.DEFAULT -> onCredentialOverride(null)
											CredentialMode.ON -> onCredentialOverride(true)
											CredentialMode.OFF -> onCredentialOverride(false)
										}
										onCredentialExpandedChange(false)
									},
								)
							}
						}
					}
				},
				description = {
					Text(
						"Credential mode",
						style = MaterialTheme.typography.bodyMedium.copy(MaterialTheme.colorScheme.onBackground),
					)
				},
				trailing = null,
			),
		),
	)
}
