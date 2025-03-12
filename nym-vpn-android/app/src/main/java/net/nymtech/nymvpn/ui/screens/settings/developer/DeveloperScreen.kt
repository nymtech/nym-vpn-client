package net.nymtech.nymvpn.ui.screens.settings.developer

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.asPaddingValues
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.systemBars
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.outlined.Bolt
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
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.derivedStateOf
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.res.vectorResource
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.AppUiState
import net.nymtech.nymvpn.ui.AppViewModel
import net.nymtech.nymvpn.ui.common.buttons.surface.SelectionItem
import net.nymtech.nymvpn.ui.common.buttons.surface.SurfaceSelectionGroupButton
import net.nymtech.nymvpn.ui.common.navigation.LocalNavController
import net.nymtech.nymvpn.ui.common.navigation.NavBarState
import net.nymtech.nymvpn.ui.common.navigation.NavIcon
import net.nymtech.nymvpn.ui.common.navigation.NavTitle
import net.nymtech.nymvpn.ui.screens.settings.developer.components.ConnectionDataDisplay
import net.nymtech.nymvpn.ui.theme.iconSize
import net.nymtech.nymvpn.util.extensions.scaledHeight
import net.nymtech.nymvpn.util.extensions.scaledWidth
import net.nymtech.vpn.backend.Tunnel

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun DeveloperScreen(appUiState: AppUiState, appViewModel: AppViewModel) {
	val navController = LocalNavController.current
	val padding = WindowInsets.systemBars.asPaddingValues()

	var environmentExpanded by remember { mutableStateOf(false) }
	var credentialExpanded by remember { mutableStateOf(false) }

	val credentialMode by remember {
		derivedStateOf {
			when (appUiState.settings.isCredentialMode) {
				true -> CredentialMode.ON
				false -> CredentialMode.OFF
				null -> CredentialMode.DEFAULT
			}
		}
	}

	LaunchedEffect(Unit) {
		appViewModel.onNavBarStateChange(
			NavBarState(
				title = { NavTitle("Developer") },
				leading = {
					NavIcon(Icons.AutoMirrored.Filled.ArrowBack) {
						navController.popBackStack()
					}
				},
			),
		)
	}

	Column(
		horizontalAlignment = Alignment.Start,
		verticalArrangement = Arrangement.spacedBy(24.dp.scaledHeight(), Alignment.Top),
		modifier =
		Modifier
			.fillMaxSize()
			.verticalScroll(rememberScrollState())
			.padding(top = 24.dp)
			.padding(horizontal = 24.dp.scaledWidth()).padding(bottom = padding.calculateBottomPadding()),
	) {
		appUiState.managerState.connectionData?.let {
			SurfaceSelectionGroupButton(
				listOf(
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
									val icon = Icons.Outlined.Bolt
									Icon(
										icon,
										icon.name,
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
						description = {
							ConnectionDataDisplay(it)
						},
						trailing = null,
					),
				),
				background = MaterialTheme.colorScheme.surface,
			)
		}
		appUiState.managerState.mixnetConnectionState?.let {
			SurfaceSelectionGroupButton(
				listOf(
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
									val icon = ImageVector.vectorResource(R.drawable.mixnet)
									Icon(
										icon,
										icon.name,
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
							Text(
								"Ipv4: ${it.ipv4State}",
								style = MaterialTheme.typography.bodyMedium.copy(MaterialTheme.colorScheme.outline),
							)
							Text(
								"Ipv6: ${it.ipv6State}",
								style = MaterialTheme.typography.bodyMedium.copy(MaterialTheme.colorScheme.outline),
							)
						},
						trailing = null,
					),
				),
				background = MaterialTheme.colorScheme.surface,
			)
		}
		SurfaceSelectionGroupButton(
			listOf(
				SelectionItem(
					{
						val icon = Icons.Outlined.Place
						Icon(
							icon,
							icon.name,
							modifier = Modifier.size(iconSize.scaledWidth()),
						)
					},
					title = {
						val scope = rememberCoroutineScope()
						ExposedDropdownMenuBox(
							expanded = environmentExpanded,
							onExpandedChange = { environmentExpanded = it },
						) {
							TextField(
								value = appUiState.settings.environment.name,
								onValueChange = {},
								readOnly = true,
								textStyle = MaterialTheme.typography.bodyMedium.copy(MaterialTheme.colorScheme.onSurface),
								trailingIcon = { ExposedDropdownMenuDefaults.TrailingIcon(expanded = environmentExpanded) },
								modifier = Modifier.menuAnchor(MenuAnchorType.PrimaryNotEditable, true),
							)
							ExposedDropdownMenu(
								expanded = environmentExpanded,
								onDismissRequest = { environmentExpanded = false },
							) {
								enumValues<Tunnel.Environment>().forEach { item ->
									DropdownMenuItem(
										text = { Text(text = item.name) },
										onClick = {
											scope.launch {
												if (appUiState.settings.environment == item) return@launch
												appViewModel.logout()
												appViewModel.onEnvironmentChange(item)
												environmentExpanded = false
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
							style = MaterialTheme.typography.bodyMedium.copy(MaterialTheme.colorScheme.outline),
						)
					},
					trailing = null,
				),
				SelectionItem(
					{
						val icon = Icons.Outlined.Key
						Icon(
							icon,
							icon.name,
							modifier = Modifier.size(iconSize.scaledWidth()),
						)
					},
					title = {
						ExposedDropdownMenuBox(
							expanded = credentialExpanded,
							onExpandedChange = { credentialExpanded = it },
						) {
							TextField(
								value = credentialMode.name,
								onValueChange = {},
								readOnly = true,
								textStyle = MaterialTheme.typography.bodyMedium.copy(MaterialTheme.colorScheme.onSurface),
								trailingIcon = { ExposedDropdownMenuDefaults.TrailingIcon(expanded = environmentExpanded) },
								modifier = Modifier.menuAnchor(MenuAnchorType.PrimaryNotEditable, true),
							)
							ExposedDropdownMenu(
								expanded = credentialExpanded,
								onDismissRequest = { credentialExpanded = false },
							) {
								enumValues<CredentialMode>().forEach { item ->
									DropdownMenuItem(
										text = { Text(text = item.name) },
										onClick = {
											if (credentialMode == item) return@DropdownMenuItem
											when (item) {
												CredentialMode.DEFAULT -> appViewModel.onCredentialOverride(null)
												CredentialMode.ON -> appViewModel.onCredentialOverride(true)
												CredentialMode.OFF -> appViewModel.onCredentialOverride(false)
											}
											credentialExpanded = false
										},
									)
								}
							}
						}
					},
					description = {
						Text(
							"Credential mode",
							style = MaterialTheme.typography.bodyMedium.copy(MaterialTheme.colorScheme.outline),
						)
					},
					trailing = null,
				),
			),
			background = MaterialTheme.colorScheme.surface,
		)
	}
}
