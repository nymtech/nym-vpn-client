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
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.asPaddingValues
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.systemBars
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.core.net.toUri
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.lifecycle.lifecycleScope
import kotlinx.coroutines.launch
import net.nymtech.connectivity.NetworkStatus
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.manager.backend.model.BackendUiEvent
import net.nymtech.nymvpn.ui.AppUiState
import net.nymtech.nymvpn.ui.AppViewModel
import net.nymtech.nymvpn.ui.MainActivity
import net.nymtech.nymvpn.ui.Route
import net.nymtech.nymvpn.ui.common.navigation.LocalNavController
import net.nymtech.nymvpn.ui.common.snackbar.AlertType
import net.nymtech.nymvpn.ui.common.snackbar.AlertAction
import net.nymtech.nymvpn.ui.common.snackbar.AlertController
import net.nymtech.nymvpn.ui.common.snackbar.AlertHost
import net.nymtech.nymvpn.ui.common.snackbar.AlertMessage
import net.nymtech.nymvpn.ui.model.ConnectionState
import net.nymtech.nymvpn.ui.screens.account.info.AutologinState
import net.nymtech.nymvpn.ui.screens.account.info.modal.AutologinLoadingDialog
import net.nymtech.nymvpn.ui.screens.account.info.modal.PinCodeDialog
import net.nymtech.nymvpn.ui.screens.auth.AuthBottomSheet
import net.nymtech.nymvpn.ui.screens.auth.AuthRoute
import net.nymtech.nymvpn.ui.screens.main.panel.ConnectAction
import net.nymtech.nymvpn.ui.screens.main.panel.ConnectMode
import net.nymtech.nymvpn.ui.screens.main.panel.ConnectPanel
import net.nymtech.nymvpn.ui.screens.main.panel.ConnectPanelState
import net.nymtech.nymvpn.ui.screens.main.panel.PanelState
import net.nymtech.nymvpn.ui.screens.main.modal.BatteryModal
import net.nymtech.nymvpn.ui.screens.main.modal.CompatibilityModal
import net.nymtech.nymvpn.ui.screens.main.modal.NetworkStatsModal
import net.nymtech.nymvpn.ui.screens.main.modal.ShowInfoModal
import net.nymtech.nymvpn.ui.screens.permission.Permission
import net.nymtech.nymvpn.ui.screens.settings.components.ExpiryState
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.util.extensions.convertSecondsToTimeString
import net.nymtech.nymvpn.util.extensions.goFromRoot
import net.nymtech.nymvpn.util.extensions.openWebUrl
import net.nymtech.nymvpn.util.extensions.savePasswordToManager
import net.nymtech.nymvpn.util.extensions.toConnectMode
import net.nymtech.nymvpn.util.extensions.toUserMessage
import net.nymtech.vpn.backend.Tunnel
import nym_vpn_lib_types.AccountControllerState
import nym_vpn_lib_types.DeeplinkKind
import androidx.compose.ui.res.stringResource
import net.nymtech.nymvpn.ui.screens.main.components.ConnectionStatus
import net.nymtech.nymvpn.ui.screens.main.panel.ServerNode
import nym_vpn_lib_types.Score

@Composable
fun MainScreen(appViewModel: AppViewModel, appUiState: AppUiState, autoStart: Boolean, showAuth: Boolean = false, viewModel: MainViewModel = hiltViewModel()) {
	val uiState = remember(appUiState.managerState, appUiState.networkStatus) {
		with(appUiState) {
			val baseState = when {
				managerState.isRestarting && networkStatus == NetworkStatus.Disconnected -> ConnectionState.Offline
				managerState.isRestarting && managerState.tunnelState == Tunnel.State.Down -> ConnectionState.Disconnecting
				managerState.isRestarting && managerState.tunnelState == Tunnel.State.InitializingClient ->
					ConnectionState.from(
						managerState.tunnelState,
						managerState.establishConnectionState,
					)

				managerState.isRestarting ->
					ConnectionState.from(
						managerState.tunnelState,
						managerState.establishConnectionState,
					)

				managerState.tunnelState !is Tunnel.State.Down && managerState.tunnelState !is Tunnel.State.Error && networkStatus == NetworkStatus.Disconnected -> ConnectionState.WaitingForConnection
				managerState.tunnelState == Tunnel.State.Down && networkStatus == NetworkStatus.Disconnected -> ConnectionState.Offline
				else ->
					ConnectionState.from(
						managerState.tunnelState,
						managerState.establishConnectionState,
					)
			}

			val finalState = when (val event = managerState.backendUiEvent) {
				is BackendUiEvent.BandwidthAlert, null -> baseState
				is BackendUiEvent.Failure -> {
					val isSubError = event.reason is nym_vpn_lib_types.ErrorStateReason.InactiveSubscription ||
						event.reason is nym_vpn_lib_types.ErrorStateReason.InactiveAccount
					val isAccountReady = managerState.accountState is AccountControllerState.ReadyToConnect ||
						managerState.accountState is AccountControllerState.Decentralised ||
						managerState.accountState is AccountControllerState.UpgradeMode
					if (isSubError && isAccountReady) {
						baseState
					} else {
						ConnectionState.Error(event.reason)
					}
				}

				is BackendUiEvent.StartFailure -> ConnectionState.StartFailure(event.exception)
			}

			MainUiState(
				connectionTime = managerState.connectionData?.connectedAt,
				connectionState = finalState,
			)
		}
	}

	val context = LocalContext.current
	val navController = LocalNavController.current
	val padding = WindowInsets.systemBars.asPaddingValues()
	var didAutoStart by rememberSaveable { mutableStateOf(false) }
	var showInfoDialog by remember { mutableStateOf(false) }
	var showCompatibilityDialog by remember { mutableStateOf(false) }
	val connectionSeconds by viewModel.connectionSeconds.collectAsState()
	var showBatteryDialog by remember { mutableStateOf(false) }
	var showNetworkStatsDialog by remember { mutableStateOf(false) }
	val autologinState by appViewModel.autologinState.collectAsState()
	val expiryBannerDismissed by viewModel.expiryBannerDismissed.collectAsState()

	var showAuthSheet by remember { mutableStateOf(false) }
	var initialAuthRoute by remember { mutableStateOf<AuthRoute>(AuthRoute.Welcome) }
	var authSheetChecked by rememberSaveable { mutableStateOf(false) }
	var isShowingConnectionErrorAlert by remember { mutableStateOf(false) }
	var isShowingExpiredAlert by remember { mutableStateOf(false) }
	var expiredAlertShown by rememberSaveable { mutableStateOf(false) }
	val activity = context as? MainActivity

	val connectionTime = remember(connectionSeconds) {
		connectionSeconds?.convertSecondsToTimeString()
	}

	with(appUiState.managerState) {
		LaunchedEffect(tunnelState, connectionData?.connectedAt, appUiState.networkStatus) {
			viewModel.onTunnelStateChanged(tunnelState, connectionData?.connectedAt, appUiState.networkStatus)
		}
	}

	LaunchedEffect(appUiState.managerState.isInitialized) {
		if (appUiState.managerState.isInitialized && !authSheetChecked) {
			authSheetChecked = true
			if (!appUiState.managerState.isMnemonicStored && !appUiState.settings.isWelcomeShown) {
				initialAuthRoute = AuthRoute.Welcome
				showAuthSheet = true
			}
		}
	}

	LaunchedEffect(showAuth) {
		if (showAuth && !appUiState.managerState.isMnemonicStored) {
			initialAuthRoute = AuthRoute.Welcome
			showAuthSheet = true
		}
	}

	fun checkBatteryOptimization() {
		val pm = context.getSystemService(PowerManager::class.java)
		val isIgnoringBatteryOptimizations = pm?.isIgnoringBatteryOptimizations(context.packageName) ?: true
		if (!isIgnoringBatteryOptimizations && !appUiState.settings.batteryDialogSkip) {
			showBatteryDialog = true
		} else {
			viewModel.onDisconnect()
		}
	}

	val alertTitle = stringResource(R.string.notification_improve_title)
	val alertAction = stringResource(R.string.notification_improve_button)

	fun checkStatsEnabled() {
		if (!appUiState.settings.statsEnabled && !appUiState.settings.statsDialogSkip) {
			AlertController.show(
				AlertMessage(
					type = AlertType.Neutral,
					title = alertTitle,
					action = AlertAction(alertAction) {
						showNetworkStatsDialog = true
					},
					duration = 7_000L,
					onDismiss = { viewModel.onNetworkStatsSkipped() },
				),
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
				viewModel.onConnect()
			}
		},
	)

	val batteryOptSettingsTitle = stringResource(R.string.battery_opt_settings_text)
	val batteryOptResultState = rememberLauncherForActivityResult(
		ActivityResultContracts.StartActivityForResult(),
		onResult = {
			val accepted = (it.resultCode == RESULT_OK)
			if (!accepted) viewModel.onBatteryOptSkipped()
			AlertController.show(AlertMessage(title = batteryOptSettingsTitle))
			viewModel.onDisconnect()
		},
	)

	val permissionAlertTitle = stringResource(R.string.notification_permission_required)
	val requestPermissionLauncher = rememberLauncherForActivityResult(
		ActivityResultContracts.RequestPermission(),
	) { isGranted ->
		if (!isGranted) {
			AlertController.show(
				AlertMessage(
					type = AlertType.Warning,
					title = permissionAlertTitle,
				),
			)
		}
	}

	fun onConnectPressed() {
		val intent = VpnService.prepare(context)
		if (intent != null) {
			vpnActivityResultState.launch(intent)
		} else {
			viewModel.onConnect()
		}
	}

	fun onDisconnectPressed() {
		checkBatteryOptimization()
		checkStatsEnabled()
	}

	fun onStopKillSwitchPressed() {
		navController.goFromRoot(Route.Settings(true))
	}

	fun onGetStartedPressed() {
		if (appUiState.subscription?.expiryState == ExpiryState.EXPIRED && appUiState.managerState.isMnemonicStored) {
			navController.goFromRoot(Route.SelectPlan)
		} else if (!appUiState.managerState.isMnemonicStored) {
			initialAuthRoute = AuthRoute.Welcome
			showAuthSheet = true
		}
	}

	val entryAlertTitle = stringResource(R.string.disabled_while_connecting)
	fun onEntryClick() {
		when (uiState.connectionState) {
			ConnectionState.WaitingForConnection -> AlertController.show(
				AlertMessage(title = entryAlertTitle),
			)

			else -> navController.goFromRoot(Route.EntryLocation)
		}
	}

	val exitAlertTitle = stringResource(R.string.disabled_while_connecting)
	fun onExitClick() {
		when (uiState.connectionState) {
			ConnectionState.WaitingForConnection -> AlertController.show(
				AlertMessage(title = exitAlertTitle),
			)

			else -> navController.goFromRoot(Route.ExitLocation)
		}
	}

	LaunchedEffect(Unit) {
		if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
			requestPermissionLauncher.launch(Manifest.permission.POST_NOTIFICATIONS)
		}
	}

	if (autoStart && !didAutoStart && uiState.connectionState is ConnectionState.Disconnected) {
		LaunchedEffect(Unit) {
			didAutoStart = true
			onConnectPressed()
		}
	}

	val autologinAlertTitle = stringResource(R.string.account_info_autologin_error)
	LaunchedEffect(autologinState) {
		if (autologinState is AutologinState.Error) {
			AlertController.show(
				AlertMessage(
					type = AlertType.Negative,
					title = autologinAlertTitle,
				),
			)
		}
	}

	val expiryAlertTitle = stringResource(R.string.banner_plan_expires_text, appUiState.subscription?.validUntilDate ?: "")
	val expiryAlertAction = stringResource(R.string.banner_renew_text)
	val expiryState = appUiState.subscription?.expiryState
	LaunchedEffect(expiryState, expiryBannerDismissed) {
		if (!expiryBannerDismissed && (expiryState == ExpiryState.WARNING)) {
			AlertController.show(
				AlertMessage(
					type = AlertType.Warning,
					title = expiryAlertTitle,
					action = AlertAction(expiryAlertAction) {
						viewModel.dismissExpiryBanner()
						appViewModel.fetchAutologin(DeeplinkKind.AUTOLOGIN_RENEW)
					},
					duration = Long.MAX_VALUE,
					onDismiss = { viewModel.dismissExpiryBanner() },
				),
			)
		}
	}

	val expiredAlertTitle = stringResource(R.string.error_expired_subscription_title)
	val expiredAlertBody = stringResource(R.string.error_expired_subscription_description)
	val expiredAlertAction = stringResource(R.string.error_expired_subscription_button)
	LaunchedEffect(expiryState) {
		if (expiryState == ExpiryState.EXPIRED && !expiredAlertShown) {
			expiredAlertShown = true
			AlertController.show(
				AlertMessage(
					type = AlertType.Error,
					title = expiredAlertTitle,
					body = expiredAlertBody,
					action = AlertAction(expiredAlertAction) { navController.goFromRoot(Route.SelectPlan) },
					duration = Long.MAX_VALUE,
					onDismiss = { isShowingExpiredAlert = false },
				),
			)
			isShowingExpiredAlert = true
		} else if (isShowingExpiredAlert && expiryState != ExpiryState.EXPIRED) {
			AlertController.dismiss()
			isShowingExpiredAlert = false
		}
	}

	val connectionErrorRetryLabel = stringResource(R.string.try_reconnecting)
	val connectionFailedLabel = stringResource(R.string.connection_failed)
	LaunchedEffect(uiState.connectionState) {
		when (val state = uiState.connectionState) {
			is ConnectionState.Error -> {
				val message = state.reason.toUserMessage(context).ifEmpty { connectionFailedLabel }
				AlertController.show(
					AlertMessage(
						type = AlertType.Error,
						title = message,
						action = AlertAction(connectionErrorRetryLabel) { onConnectPressed() },
						duration = Long.MAX_VALUE,
						onDismiss = { isShowingConnectionErrorAlert = false },
					),
				)
				isShowingConnectionErrorAlert = true
			}
			is ConnectionState.StartFailure -> {
				val message = state.exception.toUserMessage(context)
				AlertController.show(
					AlertMessage(
						type = AlertType.Error,
						title = message,
						action = AlertAction(connectionErrorRetryLabel) { onConnectPressed() },
						duration = Long.MAX_VALUE,
						onDismiss = { isShowingConnectionErrorAlert = false },
					),
				)
				isShowingConnectionErrorAlert = true
			}
			else -> if (isShowingConnectionErrorAlert) AlertController.dismiss()
		}
	}

	when (val autologin = autologinState) {
		is AutologinState.Loading -> AutologinLoadingDialog(onCancel = appViewModel::cancelAutologin)
		is AutologinState.PinReady -> PinCodeDialog(
			pinCode = autologin.pinCode,
			url = autologin.url,
			onDismiss = appViewModel::dismissAutologin,
		)

		else -> Unit
	}

	MainScreenContent(
		connectionState = uiState.connectionState,
		appUiState = appUiState,
		connectionTime = connectionTime,
		initialPanelState = if (appUiState.settings.panelCollapsed) PanelState.COLLAPSED else PanelState.FULL,
		onAction = { action ->
			when (action) {
				ConnectAction.CONNECT -> onConnectPressed()
				ConnectAction.DISCONNECT -> onDisconnectPressed()
				ConnectAction.STOP_KILL_SWITCH -> onStopKillSwitchPressed()
				ConnectAction.GET_STARTED -> onGetStartedPressed()
			}
		},
		onModeChange = { mode ->
			when (mode) {
				ConnectMode.AUTO -> viewModel.onAutoSelected()
				ConnectMode.FAST -> viewModel.onTwoHopSelected()
				ConnectMode.MIXNET -> viewModel.onFiveHopSelected()
			}
		},
		onPanelStateChange = { viewModel.onPanelStateChanged(it) },
		contentPadding = padding,
		onExitNodeClick = { onExitClick() },
		onEntryNodeClick = { onEntryClick() },
		onExitInfoClick = {
			appUiState.exitPointGateway?.identity?.let {
				navController.goFromRoot(Route.ServerDetails(it, "EXIT"))
			}
		},
		onEntryInfoClick = {
			appUiState.entryPointGateway?.identity?.let {
				navController.goFromRoot(Route.ServerDetails(it, "ENTRY"))
			}
		},
	)

	ShowInfoModal(
		context = context,
		showInfoDialog = showInfoDialog,
		onDismiss = { showInfoDialog = false },
	)

	val downloadUrl = stringResource(R.string.download_url)
	CompatibilityModal(
		showCompatibilityDialog = showCompatibilityDialog,
		onDismiss = { showCompatibilityDialog = false },
		onConfirmClick = {
			showCompatibilityDialog = false
			context.openWebUrl(downloadUrl)
		},
	)

	val batteryOptTitle = stringResource(R.string.battery_opt_settings_text)
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
			AlertController.show(
				AlertMessage(
					type = AlertType.Neutral,
					title = batteryOptTitle,
				),
			)
			viewModel.onDisconnect()
		},
	)

	val statsAlertTitle = stringResource(R.string.notification_stats_enabled)
	NetworkStatsModal(
		showNetworkStatsDialog = showNetworkStatsDialog,
		onConfirm = {
			showNetworkStatsDialog = false
			viewModel.setNetworkStatsEnabled()
			AlertController.show(
				AlertMessage(
					type = AlertType.Confirmation,
					title = statsAlertTitle,
				),
			)
		},
		onDismiss = {
			viewModel.onNetworkStatsSkipped()
			showNetworkStatsDialog = false
		},
	)

	AuthBottomSheet(
		isVisible = showAuthSheet,
		isMnemonicStored = appUiState.managerState.isMnemonicStored,
		initialRoute = initialAuthRoute,
		onDismissRequest = {
			if (!appUiState.settings.isWelcomeShown) {
				appViewModel.setWelcomeShown()
			}
			showAuthSheet = false
		},
		onAuthSuccess = {
			showAuthSheet = false
		},
		onSaveToPasswordManager = {
			activity?.lifecycleScope?.launch {
				savePasswordToManager(context = context, password = it)
			}
		},
	)
}

@Composable
private fun MainScreenContent(
	connectionState: ConnectionState,
	appUiState: AppUiState,
	connectionTime: String?,
	initialPanelState: PanelState,
	onAction: (ConnectAction) -> Unit,
	onModeChange: (ConnectMode) -> Unit,
	onExitNodeClick: () -> Unit,
	onEntryNodeClick: () -> Unit,
	onExitInfoClick: () -> Unit,
	onEntryInfoClick: () -> Unit,
	onPanelStateChange: (PanelState) -> Unit,
	modifier: Modifier = Modifier,
	contentPadding: PaddingValues = PaddingValues(),
	previewAlertMessage: AlertMessage? = null,
) {
	val connectMode = appUiState.vpnConfig.algorithm.toConnectMode(appUiState.vpnConfig.mode)
	val panelState = ConnectPanelState(
		connectionState = connectionState,
		accountState = appUiState.managerState.accountState,
		isMnemonicStored = appUiState.managerState.isMnemonicStored,
		connectMode = connectMode,
		exitNode = ServerNode(
			id = appUiState.exitPointGateway?.identity ?: "",
			name = appUiState.exitPointName,
			countryCode = appUiState.exitPointCountry,
			location = appUiState.exitPointLocation,
			score = appUiState.exitPointGateway?.wgScore ?: Score.HIGH,
		),
		entryNode = ServerNode(
			id = appUiState.entryPointGateway?.identity ?: "",
			name = appUiState.entryPointName,
			countryCode = appUiState.entryPointCountry,
			location = appUiState.entryPointLocation,
			score = appUiState.entryPointGateway?.wgScore ?: Score.HIGH,
		),
		exitIsAutoBest = connectMode == ConnectMode.AUTO &&
			appUiState.vpnConfig.algorithm == nym_vpn_lib_types.GatewaySelectionAlgorithm.AUTO &&
			appUiState.isExitPointRandom,
		initialPanelState = initialPanelState,
		isSubscriptionExpired = appUiState.subscription?.expiryState == ExpiryState.EXPIRED,
	)

	Box(
		modifier = modifier
			.fillMaxSize()
			.padding(bottom = contentPadding.calculateBottomPadding()),
	) {
		Box(
			modifier = Modifier
				.fillMaxWidth()
				.fillMaxHeight(0.40f)
				.align(Alignment.TopCenter),
			contentAlignment = Alignment.BottomCenter,
		) {
			ConnectionStatus(
				connectionState = connectionState,
				vpnMode = appUiState.vpnConfig.mode,
				establishConnectionState = appUiState.managerState.establishConnectionState,
				connectionTime = connectionTime,
			)
		}

		Surface(
			color = MaterialTheme.colorScheme.background,
			modifier = Modifier
				.align(Alignment.BottomCenter)
				.fillMaxWidth()
				.padding(horizontal = 16.dp)
				.padding(bottom = 20.dp, top = 8.dp),
		) {
			ConnectPanel(
				state = panelState,
				onModeChange = onModeChange,
				onAction = onAction,
				onPanelStateChange = onPanelStateChange,
				onEntryNodeClick = onEntryNodeClick,
				onExitNodeClick = onExitNodeClick,
				onExitInfoClick = onExitInfoClick,
				onEntryInfoClick = onEntryInfoClick,
			)
		}

		AlertHost(
			modifier = Modifier
				.align(Alignment.TopCenter)
				.padding(8.dp)
				.padding(horizontal = 16.dp),
			previewMessage = previewAlertMessage,
		)
	}
}

@Composable
@Preview(showBackground = true, backgroundColor = 0xFF0D0D0F)
private fun MainScreenPreviewAlertCritical() {
	NymVPNTheme(Theme.DARK_MODE) {
		MainScreenContent(
			connectionState = ConnectionState.Disconnected,
			appUiState = AppUiState(),
			connectionTime = null,
			initialPanelState = PanelState.FULL,
			onAction = {},
			onModeChange = {},
			onPanelStateChange = {},
			onExitNodeClick = {},
			onEntryNodeClick = {},
			onExitInfoClick = {},
			onEntryInfoClick = {},
			previewAlertMessage = AlertMessage(
				type = AlertType.Error,
				title = "Secure your secret passphrase",
				body = "No passphrase no access to account",
				action = AlertAction("Back up now") {},
				duration = Long.MAX_VALUE,
			),
		)
	}
}

@Composable
@Preview(showBackground = true, backgroundColor = 0xFF0D0D0F)
private fun MainScreenPreviewDisconnected() {
	NymVPNTheme(Theme.DARK_MODE) {
		MainScreenContent(
			connectionState = ConnectionState.Disconnected,
			appUiState = AppUiState(),
			connectionTime = null,
			initialPanelState = PanelState.FULL,
			onAction = {},
			onModeChange = {},
			onPanelStateChange = {},
			onExitNodeClick = {},
			onEntryNodeClick = {},
			onExitInfoClick = {},
			onEntryInfoClick = {},
		)
	}
}

@Composable
@Preview(showBackground = true, backgroundColor = 0xFF0D0D0F)
private fun MainScreenPreviewConnected() {
	NymVPNTheme(Theme.DARK_MODE) {
		MainScreenContent(
			connectionState = ConnectionState.Connected,
			appUiState = AppUiState(),
			connectionTime = "01:23:45",
			initialPanelState = PanelState.FULL,
			onAction = {},
			onModeChange = {},
			onPanelStateChange = {},
			onExitNodeClick = {},
			onEntryNodeClick = {},
			onExitInfoClick = {},
			onEntryInfoClick = {},
		)
	}
}
