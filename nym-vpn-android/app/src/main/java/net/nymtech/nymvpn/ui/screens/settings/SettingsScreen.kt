package net.nymtech.nymvpn.ui.screens.settings

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.asPaddingValues
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.systemBars
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.setValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalClipboardManager
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.AppUiState
import net.nymtech.nymvpn.ui.AppViewModel
import net.nymtech.nymvpn.ui.Route
import net.nymtech.nymvpn.ui.common.navigation.LocalNavController
import net.nymtech.nymvpn.ui.common.navigation.NavBarState
import net.nymtech.nymvpn.ui.common.navigation.NavIcon
import net.nymtech.nymvpn.ui.common.navigation.NavTitle
import net.nymtech.nymvpn.ui.common.snackbar.SnackbarController
import net.nymtech.nymvpn.ui.screens.settings.components.AccountId
import net.nymtech.nymvpn.ui.screens.settings.components.AccountSection
import net.nymtech.nymvpn.ui.screens.settings.components.AppVersion
import net.nymtech.nymvpn.ui.screens.settings.components.AppearanceSection
import net.nymtech.nymvpn.ui.screens.settings.components.LegalSection
import net.nymtech.nymvpn.ui.screens.settings.components.LoginSection
import net.nymtech.nymvpn.ui.screens.settings.components.LogoutDialog
import net.nymtech.nymvpn.ui.screens.settings.components.LogoutSection
import net.nymtech.nymvpn.ui.screens.settings.components.SupportSection
import net.nymtech.nymvpn.ui.screens.settings.components.VpnSettingsSection
import net.nymtech.nymvpn.util.extensions.scaledWidth
import net.nymtech.vpn.backend.Tunnel

@Composable
fun SettingsScreen(appViewModel: AppViewModel, appUiState: AppUiState, viewModel: SettingsViewModel = hiltViewModel()) {
	val context = LocalContext.current
	val snackbar = SnackbarController.current
	val navController = LocalNavController.current
	val clipboardManager = LocalClipboardManager.current
	val padding = WindowInsets.systemBars.asPaddingValues()

	var loggingOut by remember { mutableStateOf(false) }
	var showLogoutDialog by remember { mutableStateOf(false) }

	LaunchedEffect(Unit) {
		appViewModel.onNavBarStateChange(
			NavBarState(
				title = { NavTitle(stringResource(R.string.settings)) },
				leading = {
					NavIcon(Icons.AutoMirrored.Filled.ArrowBack, stringResource(R.string.back)) {
						navController.popBackStack()
					}
				},
			),
		)
	}

	LaunchedEffect(appUiState.managerState.isMnemonicStored) {
		loggingOut = false
	}

	LogoutDialog(
		show = showLogoutDialog,
		onDismiss = { showLogoutDialog = false },
		onConfirm = {
			appViewModel.logout()
			showLogoutDialog = false
			loggingOut = true
		},
	)

	Column(
		horizontalAlignment = Alignment.Start,
		verticalArrangement = Arrangement.spacedBy(24.dp, Alignment.Top),
		modifier = Modifier
			.verticalScroll(rememberScrollState())
			.fillMaxSize()
			.padding(top = 24.dp)
			.padding(horizontal = 24.dp.scaledWidth())
			.padding(bottom = padding.calculateBottomPadding()),
	) {
		LoginSection(
			appUiState = appUiState,
			onLoginClick = { navController.navigate(Route.Login) },
		)
		AccountSection(appUiState = appUiState, context = context)
		SupportSection(navController = navController)
		VpnSettingsSection(appUiState = appUiState, viewModel = viewModel, context = context)
		AppearanceSection(appUiState = appUiState, viewModel = viewModel, context = context)
		LegalSection()
		LogoutSection(
			appUiState,
			loggingOut = loggingOut,
			onLogoutClick = {
				if (appUiState.managerState.tunnelState != Tunnel.State.Down) {
					snackbar.showMessage(context.getString(R.string.action_requires_tunnel_down))
				} else {
					showLogoutDialog = true
				}
			},
		)
		if (appUiState.managerState.accountId != null) {
			AccountId(clipboardManager, appUiState.managerState.accountId)
		}
		AppVersion(clipboardManager, navController)
	}
}
