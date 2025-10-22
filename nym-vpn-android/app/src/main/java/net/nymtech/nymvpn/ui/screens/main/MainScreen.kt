package net.nymtech.nymvpn.ui.screens.main

import android.Manifest
import android.app.Activity.RESULT_OK
import android.content.Intent
import android.net.VpnService
import android.os.Build
import android.os.PowerManager
import android.provider.Settings
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.asPaddingValues
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.systemBars
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Close
import androidx.compose.material3.SnackbarDuration
import androidx.compose.material3.SnackbarHost
import androidx.compose.material3.SnackbarHostState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import androidx.core.net.toUri
import androidx.hilt.navigation.compose.hiltViewModel
import net.nymtech.connectivity.NetworkStatus
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.manager.backend.model.BackendUiEvent
import net.nymtech.nymvpn.ui.AppUiState
import net.nymtech.nymvpn.ui.Route
import net.nymtech.nymvpn.ui.common.labels.GroupLabel
import net.nymtech.nymvpn.ui.common.navigation.LocalNavController
import net.nymtech.nymvpn.ui.common.snackbar.IconAction
import net.nymtech.nymvpn.ui.common.snackbar.SnackbarAction
import net.nymtech.nymvpn.ui.common.snackbar.SnackbarController
import net.nymtech.nymvpn.ui.model.ConnectionState
import net.nymtech.nymvpn.ui.model.StateMessage.Error
import net.nymtech.nymvpn.ui.model.StateMessage.StartError
import net.nymtech.nymvpn.ui.screens.main.components.ConnectionButton
import net.nymtech.nymvpn.ui.screens.main.components.ConnectionStatus
import net.nymtech.nymvpn.ui.screens.main.components.LocationField
import net.nymtech.nymvpn.ui.screens.main.components.ModeSelector
import net.nymtech.nymvpn.ui.screens.main.modal.BatteryModal
import net.nymtech.nymvpn.ui.screens.main.modal.CompatibilityModal
import net.nymtech.nymvpn.ui.screens.main.modal.NetworkStatsModal
import net.nymtech.nymvpn.ui.screens.main.modal.ShowInfoModal
import net.nymtech.nymvpn.ui.screens.permission.Permission
import net.nymtech.nymvpn.ui.theme.Theme
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
				else -> ConnectionState.from(managerState.tunnelState, managerState.establishConnectionState)
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
	val connectionTime by viewModel.connectionTime.collectAsState()
	var showBatteryDialog by remember { mutableStateOf(false) }
	var showNetworkStatsDialog by remember { mutableStateOf(false) }
	val isAppInForeground by viewModel.isAppInForeground.collectAsState()

	with(appUiState.managerState) {
		LaunchedEffect(tunnelState, connectionData?.connectedAt) {
			viewModel.onTunnelStateChanged(tunnelState, connectionData?.connectedAt)
		}
	}

	fun checkBatteryOptimization() {
		val pm = context.getSystemService(PowerManager::class.java)
		val isIgnoringBatteryOptimizations = pm?.isIgnoringBatteryOptimizations(context.packageName) ?: true
		if (!isIgnoringBatteryOptimizations && !appUiState.settings.batteryDialogSkip) {
			showBatteryDialog = true
		} else {
			viewModel.onConnect()
		}
	}

	fun checkStatsEnabled() {
		if (!appUiState.settings.statsEnabled && !appUiState.settings.statsDialogSkip) {
			SnackbarController.showMessage(
				message = context.getString(R.string.notification_improve_title),
				action = SnackbarAction(title = context.getString(R.string.notification_improve_button)) {
					showNetworkStatsDialog = true
				},
				duration = SnackbarDuration.Long,
				iconAction = IconAction(icon = Icons.Filled.Close) {
					viewModel.onNetworkStatsSkipped()
				},
			)
		}
	}

	val vpnActivityResultState = rememberLauncherForActivityResult(
		ActivityResultContracts.StartActivityForResult(),
		onResult = {
			val accepted = (it.resultCode == RESULT_OK)
			if (!accepted) {
				navController.goFromRoot(Route.Permission(Permission.VPN))
			} else {
				checkBatteryOptimization()
			}
		},
	)

	val batteryOptResultState = rememberLauncherForActivityResult(
		ActivityResultContracts.StartActivityForResult(),
		onResult = {
			val accepted = (it.resultCode == RESULT_OK)
			if (!accepted) {
				viewModel.onBatteryOptSkipped()
			}
			snackbar.showMessage(context.getString(R.string.battery_opt_settings_text))
			viewModel.onConnect()
		},
	)

	val requestPermissionLauncher = rememberLauncherForActivityResult(
		ActivityResultContracts.RequestPermission(),
	) { isGranted ->
		if (!isGranted) snackbar.showMessage(context.getString(R.string.notification_permission_required))
	}

	fun onConnectPressed() {
		val intent = VpnService.prepare(context)
		if (intent != null) {
			vpnActivityResultState.launch(intent)
		} else {
			checkBatteryOptimization()
		}
	}

	fun onDisconnectPressed() {
		viewModel.onDisconnect()
		checkStatsEnabled()
	}

	fun onStopKillSwitchPressed() {
		navController.goFromRoot(Route.Settings(true))
	}

	fun onGetStartedPressed() {
		navController.goFromRoot(Route.SelectPlan)
	}

	LaunchedEffect(Unit) {
		if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
			requestPermissionLauncher.launch(Manifest.permission.POST_NOTIFICATIONS)
		}
	}

	if (autoStart && !didAutoStart) {
		LaunchedEffect(Unit) {
			didAutoStart = true
			onConnectPressed()
		}
	}

	Column(
		verticalArrangement = Arrangement.spacedBy(8.dp.scaledHeight(), Alignment.Top),
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
			isAppInForeground = isAppInForeground,
		)
		Spacer(modifier = Modifier.weight(1f))
		Column(
			verticalArrangement = Arrangement.spacedBy(36.dp.scaledHeight(), Alignment.Bottom),
			horizontalAlignment = Alignment.CenterHorizontally,
			modifier = Modifier
				.fillMaxSize()
				.padding(bottom = 24.dp.scaledHeight()),
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
					gatewayLocation = appUiState.entryPointLocation,
					onClick = { navController.goFromRoot(Route.EntryLocation) },
					enabled = uiState.connectionState in listOf(ConnectionState.Disconnected, ConnectionState.Offline),
				)
				LocationField(
					value = appUiState.exitPointName,
					label = stringResource(R.string.exit),
					countryCode = appUiState.exitPointCountry,
					gatewayLocation = appUiState.exitPointLocation,
					onClick = { navController.goFromRoot(Route.ExitLocation) },
					enabled = uiState.connectionState in listOf(ConnectionState.Disconnected, ConnectionState.Offline),
				)
			}
			ConnectionButton(
				connectionState = uiState.connectionState,
				stateMessage = uiState.stateMessage,
				isMnemonicStored = appUiState.managerState.isMnemonicStored,
				onConnect = { onConnectPressed() },
				onDisconnect = { onDisconnectPressed() },
				onStopKillSwitch = { onStopKillSwitchPressed() },
				onGetStart = { onGetStartedPressed() },
				navController = navController,
				snackbar = snackbar,
			)
		}
	}

	ShowInfoModal(
		context = context,
		showInfoDialog = showInfoDialog,
		onDismiss = { showInfoDialog = false },
	)

	CompatibilityModal(
		showCompatibilityDialog = showCompatibilityDialog,
		onDismiss = { showCompatibilityDialog = false },
		onConfirmClick = {
			showCompatibilityDialog = false
			context.openWebUrl(context.getString(R.string.download_url))
		},
	)

	BatteryModal(
		showBatteryDialog = showBatteryDialog,
		onClickSettings = {
			val packageName = "package:${context.packageName}".toUri()
			val intent = Intent(Settings.ACTION_REQUEST_IGNORE_BATTERY_OPTIMIZATIONS).apply {
				data = packageName
			}
			batteryOptResultState.launch(intent)
			showBatteryDialog = false
		},
		onDismiss = {
			showBatteryDialog = false
			viewModel.onBatteryOptSkipped()
			snackbar.showMessage(context.getString(R.string.battery_opt_settings_text))
			viewModel.onConnect()
		},
	)

	NetworkStatsModal(
		showNetworkStatsDialog = showNetworkStatsDialog,
		onConfirm = {
			showNetworkStatsDialog = false
			viewModel.setNetworkStatsEnabled()
			snackbar.showMessage(context.getString(R.string.notification_stats_enabled))
		},
		onDismiss = {
			viewModel.onNetworkStatsSkipped()
			showNetworkStatsDialog = false
		},
	)
}
