package net.nymtech.nymvpn.ui.screens.main

import android.app.Activity.RESULT_OK
import android.net.VpnService
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.asPaddingValues
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.systemBars
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.SnackbarHost
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import kotlinx.coroutines.delay
import net.nymtech.connectivity.NetworkStatus
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.manager.backend.model.BackendUiEvent
import net.nymtech.nymvpn.ui.AppUiState
import net.nymtech.nymvpn.ui.Route
import net.nymtech.nymvpn.ui.common.Modal
import net.nymtech.nymvpn.ui.common.buttons.MainStyledButton
import net.nymtech.nymvpn.ui.common.labels.GroupLabel
import net.nymtech.nymvpn.ui.common.navigation.LocalNavController
import net.nymtech.nymvpn.ui.common.snackbar.SnackbarController
import net.nymtech.nymvpn.ui.model.ConnectionState
import net.nymtech.nymvpn.ui.model.StateMessage.Error
import net.nymtech.nymvpn.ui.model.StateMessage.StartError
import net.nymtech.nymvpn.ui.screens.main.components.ConnectionButton
import net.nymtech.nymvpn.ui.screens.main.components.ConnectionStatus
import net.nymtech.nymvpn.ui.screens.main.components.LocationField
import net.nymtech.nymvpn.ui.screens.main.components.ModeModalBody
import net.nymtech.nymvpn.ui.screens.main.components.ModeSelector
import net.nymtech.nymvpn.ui.screens.permission.Permission
import net.nymtech.nymvpn.ui.theme.CustomTypography
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.util.extensions.convertSecondsToTimeString
import net.nymtech.nymvpn.util.extensions.goFromRoot
import net.nymtech.nymvpn.util.extensions.openWebUrl
import net.nymtech.nymvpn.util.extensions.scaledHeight
import net.nymtech.nymvpn.util.extensions.scaledWidth
import net.nymtech.vpn.backend.Tunnel

@Composable
fun MainScreen(appUiState: AppUiState, autoStart: Boolean, viewModel: MainViewModel = hiltViewModel()) {
	val uiState = remember(appUiState.managerState, appUiState.networkStatus) {
		with(appUiState) {
			val connectionState = when {
				managerState.tunnelState != Tunnel.State.Down && networkStatus == NetworkStatus.Disconnected -> ConnectionState.WaitingForConnection
				managerState.tunnelState == Tunnel.State.Down && networkStatus == NetworkStatus.Disconnected -> ConnectionState.Offline
				else -> ConnectionState.from(managerState.tunnelState)
			}
			val stateMessage = when (val event = managerState.backendUiEvent) {
				is BackendUiEvent.BandwidthAlert, null -> connectionState.stateMessage
				is BackendUiEvent.Failure -> Error(event.reason)
				is BackendUiEvent.StartFailure -> StartError(event.exception)
			}
			MainUiState(
				connectionTime = managerState.connectionData?.connectedAt,
				connectionState = connectionState,
				stateMessage = stateMessage,
			)
		}
	}

	val context = LocalContext.current
	val navController = LocalNavController.current
	val snackbar = SnackbarController.current
	val padding = WindowInsets.systemBars.asPaddingValues()
	val screenSnackbar = remember { SnackbarHostState() }
	var didAutoStart by remember { mutableStateOf(false) }
	var showInfoDialog by remember { mutableStateOf(false) }
	var showCompatibilityDialog by remember { mutableStateOf(false) }
	var connectionTime: String? by remember { mutableStateOf(null) }

	with(appUiState.managerState) {
		LaunchedEffect(tunnelState) {
			while (tunnelState == Tunnel.State.Up && connectionData != null) {
				connectionData.connectedAt?.let {
					connectionTime = (System.currentTimeMillis() / 1000L - it).convertSecondsToTimeString()
					delay(1000)
				}
			}
			connectionTime = null
		}
		LaunchedEffect(isNetworkCompatible) {
			if (!isNetworkCompatible) showCompatibilityDialog = true
		}
	}

	val vpnActivityResultState = rememberLauncherForActivityResult(
		ActivityResultContracts.StartActivityForResult(),
		onResult = {
			val accepted = (it.resultCode == RESULT_OK)
			if (!accepted) {
				navController.goFromRoot(Route.Permission(Permission.VPN))
			} else {
				viewModel.onConnect()
			}
		},
	)

	fun onConnectPressed() {
		val intent = VpnService.prepare(context)
		if (intent != null) {
			vpnActivityResultState.launch(intent)
		} else {
			viewModel.onConnect()
		}
	}

	if (autoStart && !didAutoStart) {
		LaunchedEffect(Unit) {
			didAutoStart = true
			onConnectPressed()
		}
	}

	Column(
		verticalArrangement = Arrangement.spacedBy(24.dp.scaledHeight(), Alignment.Top),
		horizontalAlignment = Alignment.CenterHorizontally,
		modifier = Modifier
			.verticalScroll(rememberScrollState())
			.fillMaxSize()
			.padding(bottom = padding.calculateBottomPadding()),
	) {
		SnackbarHost(hostState = screenSnackbar, Modifier)
		ConnectionStatus(
			connectionState = uiState.connectionState,
			vpnMode = appUiState.settings.vpnMode,
			stateMessage = uiState.stateMessage,
			connectionTime = connectionTime,
			theme = appUiState.settings.theme ?: Theme.AUTOMATIC,
		)
		Spacer(modifier = Modifier.weight(1f))
		Column(
			verticalArrangement = Arrangement.spacedBy(36.dp.scaledHeight(), Alignment.Bottom),
			horizontalAlignment = Alignment.CenterHorizontally,
			modifier = Modifier.fillMaxSize().padding(bottom = 24.dp.scaledHeight()),
		) {
			ModeSelector(
				vpnMode = appUiState.settings.vpnMode,
				connectionState = uiState.connectionState,
				onTwoHopClick = { viewModel.onTwoHopSelected() },
				onFiveHopClick = { viewModel.onFiveHopSelected() },
				onInfoClick = { showInfoDialog = true },
				snackbar = snackbar,
			)
			Column(
				verticalArrangement = Arrangement.spacedBy(24.dp.scaledHeight(), Alignment.Bottom),
				modifier = Modifier.padding(horizontal = 24.dp.scaledWidth()),
			) {
				GroupLabel(title = stringResource(R.string.connect_to))
				LocationField(
					value = appUiState.entryPointName,
					label = stringResource(R.string.entry),
					countryCode = appUiState.entryPointCountry,
					onClick = { navController.goFromRoot(Route.EntryLocation) },
					enabled = uiState.connectionState in listOf(ConnectionState.Disconnected, ConnectionState.Offline),
				)
				LocationField(
					value = appUiState.exitPointName,
					label = stringResource(R.string.exit),
					countryCode = appUiState.exitPointCountry,
					onClick = { navController.goFromRoot(Route.ExitLocation) },
					enabled = uiState.connectionState in listOf(ConnectionState.Disconnected, ConnectionState.Offline),
				)
			}
			ConnectionButton(
				connectionState = uiState.connectionState,
				isMnemonicStored = appUiState.managerState.isMnemonicStored,
				onConnect = { onConnectPressed() },
				onDisconnect = { viewModel.onDisconnect() },
				navController = navController,
				snackbar = snackbar,
			)
		}
	}

	Modal(
		show = showInfoDialog,
		onDismiss = { showInfoDialog = false },
		title = {
			Text(
				text = stringResource(R.string.mode_selection).uppercase(),
				color = MaterialTheme.colorScheme.onSurface,
				style = CustomTypography.labelHuge,
				fontFamily = FontFamily(Font(R.font.lab_grotesque_mono)),
			)
		},
		text = {
			ModeModalBody(
				onClick = { context.openWebUrl(context.getString(R.string.mode_support_link)) },
			)
		},
	)

	Modal(
		show = showCompatibilityDialog,
		onDismiss = { showCompatibilityDialog = false },
		title = {
			Text(
				text = stringResource(R.string.update_required).uppercase(),
				color = MaterialTheme.colorScheme.onSurface,
				style = CustomTypography.labelHuge,
				fontFamily = FontFamily(Font(R.font.lab_grotesque_mono)),
			)
		},
		text = {
			Column(verticalArrangement = Arrangement.spacedBy(16.dp.scaledHeight())) {
				Row(
					horizontalArrangement = Arrangement.spacedBy(10.dp.scaledWidth(), Alignment.CenterHorizontally),
					verticalAlignment = Alignment.CenterVertically,
				) {
					Text(
						text = stringResource(R.string.app_update_required),
						style = MaterialTheme.typography.bodyMedium,
						color = MaterialTheme.colorScheme.onSurface,
					)
				}
			}
		},
		confirmButton = {
			MainStyledButton(
				onClick = {
					showCompatibilityDialog = false
					context.openWebUrl(context.getString(R.string.download_url))
				},
				content = { Text(stringResource(R.string.update).uppercase(), fontFamily = FontFamily(Font(R.font.lab_grotesque_mono))) },
				modifier = Modifier.fillMaxWidth().height(56.dp.scaledHeight()),
			)
		},
	)
}
