package net.nymtech.nymvpn.ui.screens.settings.environment

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.outlined.AdminPanelSettings
import androidx.compose.material.icons.outlined.AirlineStops
import androidx.compose.material.icons.outlined.Edit
import androidx.compose.material.icons.outlined.Key
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TextField
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
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
import net.nymtech.nymvpn.ui.common.buttons.IconSurfaceButton
import net.nymtech.nymvpn.ui.common.buttons.MainStyledButton
import net.nymtech.nymvpn.ui.common.buttons.ScaledSwitch
import net.nymtech.nymvpn.ui.common.buttons.surface.SelectionItem
import net.nymtech.nymvpn.ui.common.buttons.surface.SurfaceSelectionGroupButton
import net.nymtech.nymvpn.ui.common.navigation.LocalNavController
import net.nymtech.nymvpn.ui.common.navigation.NavBarState
import net.nymtech.nymvpn.ui.common.navigation.NavIcon
import net.nymtech.nymvpn.ui.common.navigation.NavTitle
import net.nymtech.nymvpn.util.extensions.navigateAndForget
import net.nymtech.nymvpn.util.extensions.scaledHeight
import net.nymtech.nymvpn.util.extensions.scaledWidth
import net.nymtech.vpn.backend.Tunnel

@Composable
fun DeveloperScreen(appUiState: AppUiState, appViewModel: AppViewModel, viewModel: DeveloperViewModel = hiltViewModel()) {
	val navController = LocalNavController.current
	val clipboardManager = LocalClipboardManager.current

	val environmentChange = viewModel.environmentChanged.collectAsStateWithLifecycle()
	var showEntryModal by remember { mutableStateOf(false) }
	var showExitModal by remember { mutableStateOf(false) }
	var gatewayId by remember { mutableStateOf("") }

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
		enumValues<Tunnel.Environment>().forEach {
			IconSurfaceButton(
				title = it.name,
				onClick = {
					if (appUiState.settings.environment == it) return@IconSurfaceButton
					appViewModel.logout()
					viewModel.onEnvironmentChange(it)
				},
				selected = appUiState.settings.environment == it,
			)
		}
		SurfaceSelectionGroupButton(
			buildList {
				addAll(
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
						SelectionItem(
							Icons.Outlined.AdminPanelSettings,
							{
								ScaledSwitch(
									appUiState.settings.isCredentialMode != null,
									onClick = { viewModel.onCredentialOverride(if (it) false else null) },
								)
							},
							title = { Text("Credential override", style = MaterialTheme.typography.bodyLarge.copy(MaterialTheme.colorScheme.onSurface)) },
							description = {
								Text(
									"Override credential defaults",
									style = MaterialTheme.typography.bodyMedium.copy(MaterialTheme.colorScheme.outline),
								)
							},
						),
					),
				)
				if (appUiState.settings.isCredentialMode != null) {
					add(
						SelectionItem(
							Icons.Outlined.Key,
							{
								ScaledSwitch(
									appUiState.settings.isCredentialMode == true,
									onClick = { viewModel.onCredentialOverride(it) },
								)
							},
							title = { Text("Credential mode enabled", style = MaterialTheme.typography.bodyLarge.copy(MaterialTheme.colorScheme.onSurface)) },
						),
					)
				}
			},
		)
	}
}
