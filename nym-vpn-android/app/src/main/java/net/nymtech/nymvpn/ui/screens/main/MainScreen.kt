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
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.core.net.toUri
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.AppUiState
import net.nymtech.nymvpn.ui.AppViewModel
import net.nymtech.nymvpn.ui.Route
import net.nymtech.nymvpn.ui.common.navigation.LocalNavController
import net.nymtech.nymvpn.ui.common.snackbar.AlertAction
import net.nymtech.nymvpn.ui.common.snackbar.AlertController
import net.nymtech.nymvpn.ui.common.snackbar.AlertHost
import net.nymtech.nymvpn.ui.common.snackbar.AlertMessage
import net.nymtech.nymvpn.ui.common.snackbar.AlertType
import net.nymtech.nymvpn.ui.model.ConnectionState
import net.nymtech.nymvpn.ui.AuthRoute
import net.nymtech.nymvpn.ui.screens.main.bottomsheet.MainBottomSheetContent
import net.nymtech.nymvpn.ui.screens.main.components.ConnectionStatus
import net.nymtech.nymvpn.ui.screens.main.panel.ConnectAction
import net.nymtech.nymvpn.ui.screens.main.panel.ConnectMode
import net.nymtech.nymvpn.ui.screens.main.panel.ConnectPanel
import net.nymtech.nymvpn.ui.screens.main.panel.ConnectPanelState
import net.nymtech.nymvpn.ui.screens.main.panel.PanelState
import net.nymtech.nymvpn.ui.screens.main.panel.ServerNode
import net.nymtech.nymvpn.ui.screens.permission.Permission
import net.nymtech.nymvpn.ui.screens.settings.components.ExpiryState
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.util.extensions.convertSecondsToTimeString
import net.nymtech.nymvpn.util.extensions.goFromRoot
import net.nymtech.nymvpn.util.extensions.openWebUrl
import net.nymtech.nymvpn.util.extensions.toConnectMode
import nym_vpn_lib_types.AccountControllerErrorStateReason
import nym_vpn_lib_types.AccountControllerState
import nym_vpn_lib_types.DeeplinkKind
import nym_vpn_lib_types.Score

@Composable
fun MainScreen(appViewModel: AppViewModel, appUiState: AppUiState, autoStart: Boolean, authRoute: AuthRoute? = null, loginProcessing: Boolean = false, viewModel: MainViewModel = hiltViewModel()) {
	val uiState by viewModel.uiState.collectAsStateWithLifecycle()
	val context = LocalContext.current
	val navController = LocalNavController.current
	val padding = WindowInsets.systemBars.asPaddingValues()

	val connectionSeconds by viewModel.connectionSeconds.collectAsStateWithLifecycle()
	val connectionTime = remember(connectionSeconds) { connectionSeconds?.convertSecondsToTimeString() }
	val autologinState by appViewModel.autologinState.collectAsStateWithLifecycle()
	val expiryBannerDismissed by viewModel.expiryBannerDismissed.collectAsStateWithLifecycle()

	var bottomSheetContent by remember { mutableStateOf<MainBottomSheetContent>(MainBottomSheetContent.Hidden) }
	var authSheetHeightPx by remember { mutableIntStateOf(0) }
	var authSheetChecked by rememberSaveable { mutableStateOf(false) }
	var didAutoStart by rememberSaveable { mutableStateOf(false) }

	var showInfoDialog by remember { mutableStateOf(false) }
	var showCompatibilityDialog by remember { mutableStateOf(false) }
	var showBatteryDialog by remember { mutableStateOf(false) }
	var showNetworkStatsDialog by remember { mutableStateOf(false) }
	var showNodeFamiliesDialog by remember { mutableStateOf(false) }

	// ── Launchers ────────────────────────────────────────────────────────────

	val vpnActivityResultState = rememberLauncherForActivityResult(
		ActivityResultContracts.StartActivityForResult(),
	) { result ->
		if (result.resultCode == RESULT_OK) {
			viewModel.onConnect()
		} else {
			navController.goFromRoot(Route.Permission(Permission.VPN))
		}
	}

	val batteryOptSettingsTitle = stringResource(R.string.battery_opt_settings_text)
	val batteryOptResultState = rememberLauncherForActivityResult(
		ActivityResultContracts.StartActivityForResult(),
	) { result ->
		if (result.resultCode != RESULT_OK) viewModel.onBatteryOptSkipped()
		AlertController.show(AlertMessage(title = batteryOptSettingsTitle))
		viewModel.onDisconnect()
	}

	val permissionAlertTitle = stringResource(R.string.notification_permission_required)
	val requestPermissionLauncher = rememberLauncherForActivityResult(
		ActivityResultContracts.RequestPermission(),
	) { isGranted ->
		if (!isGranted) {
			AlertController.show(AlertMessage(type = AlertType.Warning, title = permissionAlertTitle))
		}
	}

	// ── Action handlers ──────────────────────────────────────────────────────

	fun onConnectPressed() {
		val intent = VpnService.prepare(context)
		if (intent != null) vpnActivityResultState.launch(intent) else viewModel.onConnect()
	}

	fun checkBatteryOptimization() {
		val pm = context.getSystemService(PowerManager::class.java)
		val ignoring = pm?.isIgnoringBatteryOptimizations(context.packageName) ?: true
		if (!ignoring && !appUiState.settings.batteryDialogSkip) {
			showBatteryDialog = true
		} else {
			viewModel.onDisconnect()
		}
	}

	val statsAlertTitle = stringResource(R.string.notification_improve_title)
	val statsAlertAction = stringResource(R.string.notification_improve_button)
	fun checkStatsEnabled() {
		if (!appUiState.settings.statsEnabled && !appUiState.settings.statsDialogSkip) {
			AlertController.show(
				AlertMessage(
					type = AlertType.Neutral,
					title = statsAlertTitle,
					action = AlertAction(statsAlertAction) { showNetworkStatsDialog = true },
					duration = 7_000L,
					onDismiss = { viewModel.onNetworkStatsSkipped() },
				),
			)
		}
	}

	val nodeAlertTitle = stringResource(R.string.disabled_while_connecting)
	fun onEntryClick() {
		if (uiState.connectionState == ConnectionState.WaitingForConnection) {
			AlertController.show(AlertMessage(title = nodeAlertTitle))
		} else {
			navController.goFromRoot(Route.EntryServer)
		}
	}

	fun onExitClick() {
		if (uiState.connectionState == ConnectionState.WaitingForConnection) {
			AlertController.show(AlertMessage(title = nodeAlertTitle))
		} else {
			navController.goFromRoot(Route.ExitServer)
		}
	}

	fun onGetStartedPressed() {
		val accountState = appUiState.managerState.accountState
		when {
			accountState is AccountControllerState.Error &&
				accountState.v1 is AccountControllerErrorStateReason.AccountStatusNotActive ->
				viewModel.registerAccount()
			appUiState.subscription?.expiryState == ExpiryState.EXPIRED && appUiState.managerState.isMnemonicStored ->
				navController.goFromRoot(Route.SelectPlan)
			!appUiState.managerState.isMnemonicStored -> {
				bottomSheetContent = MainBottomSheetContent.Auth(AuthRoute.Welcome)
			}
		}
	}

	// ── Effects ──────────────────────────────────────────────────────────────

	LaunchedEffect(appUiState.managerState.isInitialized) {
		if (appUiState.managerState.isInitialized && !authSheetChecked) {
			authSheetChecked = true
			if (!appUiState.managerState.isMnemonicStored && !appUiState.settings.isWelcomeShown) {
				bottomSheetContent = MainBottomSheetContent.Auth(AuthRoute.Welcome)
			}
		}
	}

	LaunchedEffect(authRoute) {
		if (authRoute == null) return@LaunchedEffect
		if (authRoute == AuthRoute.TechOpt || !appUiState.managerState.isMnemonicStored) {
			bottomSheetContent = MainBottomSheetContent.Auth(authRoute)
		}
	}

	LaunchedEffect(loginProcessing) {
		if (loginProcessing) {
			bottomSheetContent = MainBottomSheetContent.LoginProcessing
		}
	}

	LaunchedEffect(Unit) {
		if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
			requestPermissionLauncher.launch(Manifest.permission.POST_NOTIFICATIONS)
		}
	}

	LaunchedEffect(Unit) {
		viewModel.events.collect { event ->
			when (event) {
				is MainUiEvent.ShowNodeFamiliesDialog -> showNodeFamiliesDialog = true
				is MainUiEvent.NavigateToSelectPlan -> navController.goFromRoot(Route.SelectPlan)
			}
		}
	}

	if (autoStart && !didAutoStart && uiState.connectionState is ConnectionState.Disconnected) {
		LaunchedEffect(Unit) {
			didAutoStart = true
			onConnectPressed()
		}
	}

	// ── Alerts ───────────────────────────────────────────────────────────────

	MainAlerts(
		connectionState = uiState.connectionState,
		accountState = appUiState.managerState.accountState,
		autologinState = autologinState,
		expiryState = appUiState.subscription?.expiryState,
		validUntilDate = appUiState.subscription?.validUntilDate ?: "",
		expiryBannerDismissed = expiryBannerDismissed,
		onRetryConnect = ::onConnectPressed,
		onDismissExpiryBanner = viewModel::dismissExpiryBanner,
		onRenewSubscription = { appViewModel.fetchAutologin(DeeplinkKind.AUTOLOGIN_RENEW) },
		onNavigateToSelectPlan = { navController.goFromRoot(Route.SelectPlan) },
	)

	// ── Content ──────────────────────────────────────────────────────────────

	MainScreenContent(
		connectionState = uiState.connectionState,
		appUiState = appUiState,
		connectionTime = connectionTime,
		initialPanelState = if (appUiState.settings.panelCollapsed) PanelState.COLLAPSED else PanelState.FULL,
		contentPadding = padding,
		onAction = { action ->
			when (action) {
				ConnectAction.CONNECT -> onConnectPressed()
				ConnectAction.DISCONNECT -> {
					checkBatteryOptimization()
					checkStatsEnabled()
				}
				ConnectAction.STOP_KILL_SWITCH -> navController.goFromRoot(Route.Settings(true))
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

	// ── Modals ───────────────────────────────────────────────────────────────

	val downloadUrl = stringResource(R.string.download_url)
	MainModals(
		autologinState = autologinState,
		showInfoDialog = showInfoDialog,
		showCompatibilityDialog = showCompatibilityDialog,
		showBatteryDialog = showBatteryDialog,
		showNetworkStatsDialog = showNetworkStatsDialog,
		showNodeFamiliesDialog = showNodeFamiliesDialog,
		bottomSheetContent = bottomSheetContent,
		onCancelAutologin = appViewModel::cancelAutologin,
		onDismissAutologin = appViewModel::dismissAutologin,
		onDismissInfo = { showInfoDialog = false },
		onDismissCompatibility = { showCompatibilityDialog = false },
		onConfirmCompatibility = {
			showCompatibilityDialog = false
			context.openWebUrl(downloadUrl)
		},
		onClickBatterySettings = {
			val packageName = "package:${context.packageName}".toUri()
			batteryOptResultState.launch(
				Intent(Settings.ACTION_REQUEST_IGNORE_BATTERY_OPTIMIZATIONS).apply { data = packageName },
			)
			showBatteryDialog = false
		},
		onDismissBattery = {
			showBatteryDialog = false
			viewModel.onBatteryOptSkipped()
			viewModel.onDisconnect()
		},
		onConfirmStats = {
			showNetworkStatsDialog = false
			viewModel.setNetworkStatsEnabled()
		},
		onDismissStats = {
			viewModel.onNetworkStatsSkipped()
			showNetworkStatsDialog = false
		},
		onConfirmNodeFamilies = {
			showNodeFamiliesDialog = false
			viewModel.onNodeFamiliesConfirm()
		},
		onDismissNodeFamilies = {
			showNodeFamiliesDialog = false
			viewModel.onNodeFamiliesCancel()
		},
		onNotificationSettingsClick = { navController.goFromRoot(Route.Notifications) },
		onDismissBottomSheet = {
			if (!appUiState.settings.isWelcomeShown) appViewModel.setWelcomeShown()
			bottomSheetContent = MainBottomSheetContent.Hidden
		},
		onAuthSuccess = { bottomSheetContent = MainBottomSheetContent.Hidden },
		onLoginProcessingStart = {
			bottomSheetContent = MainBottomSheetContent.LoginProcessing
		},
		authSheetMinHeightPx = authSheetHeightPx,
		onAuthSheetHeightChange = { height -> authSheetHeightPx = height },
		appUiState = appUiState,
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
	val connectMode = appUiState.vpnConfig.mode.toConnectMode()
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
		exitIsAutoBest = connectMode == ConnectMode.AUTO && appUiState.isExitPointRandom,
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
				title = stringResource(R.string.error_inactive_account),
				body = stringResource(R.string.error_inactive_account_subtitle),
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
