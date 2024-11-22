package net.nymtech.nymvpn.ui.screens.settings.developer

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.outlined.AdminPanelSettings
import androidx.compose.material.icons.outlined.AirlineStops
import androidx.compose.material.icons.outlined.Bolt
import androidx.compose.material.icons.outlined.Edit
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
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalClipboardManager
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import net.nymtech.nymvpn.ui.AppUiState
import net.nymtech.nymvpn.ui.AppViewModel
import net.nymtech.nymvpn.ui.Route
import net.nymtech.nymvpn.ui.common.Modal
import net.nymtech.nymvpn.ui.common.buttons.MainStyledButton
import net.nymtech.nymvpn.ui.common.buttons.ScaledSwitch
import net.nymtech.nymvpn.ui.common.buttons.surface.SelectionItem
import net.nymtech.nymvpn.ui.common.buttons.surface.SurfaceSelectionGroupButton
import net.nymtech.nymvpn.ui.common.navigation.LocalNavController
import net.nymtech.nymvpn.ui.common.navigation.NavBarState
import net.nymtech.nymvpn.ui.common.navigation.NavIcon
import net.nymtech.nymvpn.ui.common.navigation.NavTitle
import net.nymtech.nymvpn.ui.theme.iconSize
import net.nymtech.nymvpn.util.extensions.navigateAndForget
import net.nymtech.nymvpn.util.extensions.scaledHeight
import net.nymtech.nymvpn.util.extensions.scaledWidth
import net.nymtech.vpn.backend.Tunnel
import nym_vpn_lib.TunnelConnectionData

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun DeveloperScreen(appUiState: AppUiState, appViewModel: AppViewModel, viewModel: DeveloperViewModel = hiltViewModel()) {
	val navController = LocalNavController.current
	val clipboardManager = LocalClipboardManager.current

	val environmentChange = viewModel.environmentChanged.collectAsStateWithLifecycle()
	var showEntryModal by remember { mutableStateOf(false) }
	var showExitModal by remember { mutableStateOf(false) }
	var gatewayId by remember { mutableStateOf("") }
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

	LaunchedEffect(environmentChange.value) {
		if (environmentChange.value) navController.navigateAndForget(Route.Main(configChange = true))
	}

	Modal(showEntryModal, { showEntryModal = false }, { Text("Entry gateway id") }, {
		TextField(gatewayId, onValueChange = { gatewayId = it })
	}, confirmButton = {
		MainStyledButton(
			onClick = {
				viewModel.onEntryGateway(gatewayId)
				gatewayId = ""
				showEntryModal = false
			},
			content = {
				Text(text = "Save")
			},
		)
	}, icon = Icons.Outlined.AirlineStops)

	Modal(showExitModal, { showExitModal = false }, { Text("Exit gateway id") }, {
		TextField(gatewayId, onValueChange = { gatewayId = it })
	}, confirmButton = {
		MainStyledButton(
			onClick = {
				viewModel.onExitGateway(gatewayId)
				gatewayId = ""
				showExitModal = false
			},
			content = {
				Text(text = "Save")
			},
		)
	}, icon = Icons.Outlined.AirlineStops)

	Column(
		horizontalAlignment = Alignment.Start,
		verticalArrangement = Arrangement.spacedBy(24.dp.scaledHeight(), Alignment.Top),
		modifier =
		Modifier
			.fillMaxSize()
			.verticalScroll(rememberScrollState())
			.padding(top = 24.dp.scaledHeight())
			.padding(horizontal = 24.dp.scaledWidth()),
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
											"Connection Details",
											style = MaterialTheme.typography.bodyMedium.copy(MaterialTheme.colorScheme.onSurface),
										)
									}
								}
							}
						},
						description = {
							Text(
								"Entry gatewayId: ${it.entryGateway}",
								style = MaterialTheme.typography.bodyMedium.copy(MaterialTheme.colorScheme.outline),
							)
							Text(
								"Exit gatewayId: ${it.exitGateway}",
								style = MaterialTheme.typography.bodyMedium.copy(MaterialTheme.colorScheme.outline),
							)
							Text(
								"Connected at: ${it.connectedAt}",
								style = MaterialTheme.typography.bodyMedium.copy(MaterialTheme.colorScheme.outline),
							)
							when (val details = it.tunnel) {
								is TunnelConnectionData.Mixnet -> {
									Text(
										"Ipv4: ${details.v1.ipv4}",
										style = MaterialTheme.typography.bodyMedium.copy(MaterialTheme.colorScheme.outline),
									)
									Text(
										"Ipv6: ${details.v1.ipv6}",
										style = MaterialTheme.typography.bodyMedium.copy(MaterialTheme.colorScheme.outline),
									)
									Text(
										"Exit IPR: ${details.v1.exitIpr}",
										style = MaterialTheme.typography.bodyMedium.copy(MaterialTheme.colorScheme.outline),
									)
									Text(
										"Nym address: ${details.v1.nymAddress}",
										style = MaterialTheme.typography.bodyMedium.copy(MaterialTheme.colorScheme.outline),
									)
								}
								is TunnelConnectionData.Wireguard -> {
									Text(
										"Entry endpoint: ${details.v1.entry.endpoint}",
										style = MaterialTheme.typography.bodyMedium.copy(MaterialTheme.colorScheme.outline),
									)
									Text(
										"Entry pub key: ${details.v1.entry.publicKey}",
										style = MaterialTheme.typography.bodyMedium.copy(MaterialTheme.colorScheme.outline),
									)
									Text(
										"Entry Ipv4: ${details.v1.entry.privateIpv4}",
										style = MaterialTheme.typography.bodyMedium.copy(MaterialTheme.colorScheme.outline),
									)
									Text(
										"Exit endpoint: ${details.v1.exit.endpoint}",
										style = MaterialTheme.typography.bodyMedium.copy(MaterialTheme.colorScheme.outline),
									)
									Text(
										"Exit pub key: ${details.v1.exit.publicKey}",
										style = MaterialTheme.typography.bodyMedium.copy(MaterialTheme.colorScheme.outline),
									)
									Text(
										"Exit Ipv4: ${details.v1.exit.privateIpv4}",
										style = MaterialTheme.typography.bodyMedium.copy(MaterialTheme.colorScheme.outline),
									)
								}
							}
						},
					),
				),
			)
		}
		SurfaceSelectionGroupButton(
			listOf(
				SelectionItem(
					Icons.Outlined.Place,
					title = {
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
											if (appUiState.settings.environment == item) return@DropdownMenuItem
											appViewModel.logout()
											viewModel.onEnvironmentChange(item)
											environmentExpanded = false
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
					trailing = {},
				),
				SelectionItem(
					Icons.Outlined.Key,
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
												CredentialMode.DEFAULT -> viewModel.onCredentialOverride(null)
												CredentialMode.ON -> viewModel.onCredentialOverride(true)
												CredentialMode.OFF -> viewModel.onCredentialOverride(false)
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
					trailing = {},
				),
			),
		)
		SurfaceSelectionGroupButton(
			listOf(
				SelectionItem(
					Icons.Outlined.AdminPanelSettings,
					{
						ScaledSwitch(
							appUiState.settings.isManualGatewayOverride,
							onClick = { viewModel.onManualGatewayOverride(it) },
						)
					},
					title = { Text("Manual gateways", style = MaterialTheme.typography.bodyLarge.copy(MaterialTheme.colorScheme.onSurface)) },
					description = {
						Text(
							"Override country selection",
							style = MaterialTheme.typography.bodyMedium.copy(MaterialTheme.colorScheme.outline),
						)
					},
				),
				SelectionItem(
					Icons.Outlined.AirlineStops,
					{
						val icon = Icons.Outlined.Edit
						Icon(icon, icon.name)
					},
					title = { Text("Entry gateway id", style = MaterialTheme.typography.bodyLarge.copy(MaterialTheme.colorScheme.onSurface)) },
					description = appUiState.settings.entryGatewayId?.let {
						{
							Text(
								it,
								style = MaterialTheme.typography.bodyMedium.copy(MaterialTheme.colorScheme.outline),
								modifier = Modifier.clickable { clipboardManager.setText(AnnotatedString(it)) },
							)
						}
					},
					onClick = {
						gatewayId = appUiState.settings.entryGatewayId ?: ""
						showEntryModal = true
					},
				),
				SelectionItem(
					Icons.Outlined.AirlineStops,
					{
						val icon = Icons.Outlined.Edit
						Icon(icon, icon.name)
					},
					title = { Text("Exit gateway id", style = MaterialTheme.typography.bodyLarge.copy(MaterialTheme.colorScheme.onSurface)) },
					description = appUiState.settings.exitGatewayId?.let {
						{
							Text(
								it,
								style = MaterialTheme.typography.bodyMedium.copy(MaterialTheme.colorScheme.outline),
								modifier = Modifier.clickable { clipboardManager.setText(AnnotatedString(it)) },
							)
						}
					},
					onClick = {
						gatewayId = appUiState.settings.exitGatewayId ?: ""
						showExitModal = true
					},
				),
			),
		)
	}
}
