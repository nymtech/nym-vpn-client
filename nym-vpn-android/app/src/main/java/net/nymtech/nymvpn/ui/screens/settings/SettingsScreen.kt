package net.nymtech.nymvpn.ui.screens.settings

import android.app.Activity
import android.content.res.Configuration
import android.widget.Toast
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.MaterialTheme
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
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import kotlinx.coroutines.flow.collectLatest
import net.nymtech.nymvpn.BuildConfig
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.AppUiState
import net.nymtech.nymvpn.ui.AppViewModel
import net.nymtech.nymvpn.ui.Route
import net.nymtech.nymvpn.ui.AuthRoute
import net.nymtech.nymvpn.ui.routeName
import net.nymtech.nymvpn.ui.common.events.UiEvent
import net.nymtech.nymvpn.ui.common.navigation.LocalNavController
import net.nymtech.nymvpn.ui.common.snackbar.AlertController
import net.nymtech.nymvpn.ui.screens.settings.components.AccountSection
import net.nymtech.nymvpn.ui.screens.settings.components.AppVersionSection
import net.nymtech.nymvpn.ui.screens.settings.components.AppearanceSection
import net.nymtech.nymvpn.ui.screens.settings.components.ExpiryState
import net.nymtech.nymvpn.ui.screens.settings.components.LegalSection
import net.nymtech.nymvpn.ui.screens.settings.components.LoginSection
import net.nymtech.nymvpn.ui.screens.settings.components.LogoutDialog
import net.nymtech.nymvpn.ui.screens.settings.components.LogoutSection
import net.nymtech.nymvpn.ui.screens.settings.components.LogsSection
import net.nymtech.nymvpn.ui.screens.settings.components.QuitSection
import net.nymtech.nymvpn.ui.screens.settings.components.SubscriptionUiState
import net.nymtech.nymvpn.ui.screens.settings.components.SupportSection
import net.nymtech.nymvpn.ui.screens.settings.components.VpnSettingsSection
import net.nymtech.nymvpn.ui.screens.settings.modal.PrivateDnsDialog
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.util.DeviceAuthHelper
import net.nymtech.nymvpn.util.extensions.goFromRoot
import net.nymtech.nymvpn.util.extensions.isPrivateDnsEnabled
import net.nymtech.nymvpn.util.extensions.launchBatteryOptSettingsScreen
import net.nymtech.nymvpn.util.extensions.launchPrivateDnsSettings
import net.nymtech.nymvpn.util.extensions.launchVpnSettings
import androidx.compose.foundation.layout.navigationBarsPadding
import net.nymtech.nymvpn.util.extensions.scaledWidth
import kotlin.Boolean

@Composable
fun SettingsScreen(appUiState: AppUiState, appViewModel: AppViewModel, showVpnSettings: Boolean = false, viewModel: SettingsViewModel = hiltViewModel()) {
	val context = LocalContext.current
	val navController = LocalNavController.current

	val uiState by viewModel.uiState.collectAsStateWithLifecycle()

	var loggingOut by remember { mutableStateOf(false) }
	var showLogoutDialog by remember { mutableStateOf(false) }
	var showPrivateDnsDialog by remember { mutableStateOf(false) }

	val shortcutsInfoText = stringResource(R.string.shortcuts_info_message)
	val lanReconnectingText = stringResource(R.string.settings_event_lan_reconnecting)
	LaunchedEffect(viewModel) {
		viewModel.events.collectLatest { event ->
			when (event) {
				UiEvent.ReconnectStarted ->
					Toast.makeText(context, lanReconnectingText, Toast.LENGTH_SHORT).show()
			}
		}
	}

	LaunchedEffect(appUiState.managerState.isMnemonicStored) {
		loggingOut = false
		showLogoutDialog = false
	}

	LogoutDialog(
		show = showLogoutDialog,
		isLoggingOut = loggingOut,
		onDismiss = { showLogoutDialog = false },
		onConfirm = {
			loggingOut = true
			AlertController.dismiss()
			appViewModel.logout {
				navController.navigate(Route.Main()) {
					popUpTo(0) { inclusive = true }
					launchSingleTop = true
				}
			}
		},
	)

	PrivateDnsDialog(
		showPrivateDnsDialog = showPrivateDnsDialog,
		onDismiss = { showPrivateDnsDialog = false },
		onClickSettings = {
			showPrivateDnsDialog = false
			context.launchPrivateDnsSettings()
		},
	)

	if (showVpnSettings) {
		LaunchedEffect(Unit) {
			context.launchVpnSettings()
		}
	}

	SettingsScreen(
		SettingsValues(
			isMnemonicStored = appUiState.managerState.isMnemonicStored,
			autoConnectEnabled = appUiState.settings.autoStartEnabled,
			bypassLanEnabled = appUiState.vpnConfig.bypassLan,
			adBlockingEnabled = appUiState.vpnConfig.adBlockingEnabled,
			supportIPv6Enabled = false,
			autoselectServerEnabled = false,
			appShortcutsEnabled = appUiState.settings.isShortcutsEnabled,
			appDeviceStartupEnabled = false,
			appSystemTrayEnabled = false,
			appVersion = BuildConfig.VERSION_NAME,
			daemonVersion = uiState.daemonVersion,
			subscription = appUiState.subscription,
		),
		SettingsActions(
			onGetStartedClick = {
				navController.goFromRoot(Route.Main(authRoute = AuthRoute.Welcome.routeName))
			},
			onAccountClick = {
				navController.navigate(Route.Account)
			},
			onPassphraseClick = {
				navController.navigate(Route.Passphrase)
			},
			onSupportClick = {
				navController.navigate(Route.Support)
			},
			onResetClick = {
			},
			onLegalClick = {
				navController.navigate(Route.Legal)
			},
			onSystemStatusClick = {
			},
			onQuitClick = {
				(context as Activity).finishAffinity()
				context.finishAndRemoveTask()
			},
			onLogoutClick = {
				showLogoutDialog = true
			},
			onAppVersionClick = {
				navController.navigate(Route.Developer)
			},
			onSplitTunnelingClick = {
				navController.navigate(Route.SplitTunneling)
			},
			onAutoConnectEnable = { viewModel.onAutoConnectSelected(it) },
			onBypassLanEnable = { viewModel.onBypassLanSelected(it) },
			onAdBlockingEnable = {
				if (it && context.isPrivateDnsEnabled()) {
					showPrivateDnsDialog = true
				}
				viewModel.onAdBlockingSelected(it)
			},
			onSupportIPv6Enable = {
			},
			onAutoselectServerEnable = {
			},
			onShortcutsEnable = { enable ->
				if (enable && !DeviceAuthHelper.isDeviceSecure(context)) {
					Toast.makeText(
						context,
						shortcutsInfoText,
						Toast.LENGTH_LONG,
					).show()
				}
				viewModel.onAppShortcutsSelected(enable)
			},
			onDeviceStartupEnable = {
			},
			onSystemTrayEnable = {
			},
			onKillSwitchClick = {
				context.launchVpnSettings()
			},
			onCensorshipClick = {
				navController.navigate(Route.Censorship)
			},
			onDnsClick = {
				navController.navigate(Route.Dns)
			},
			onAppearanceClick = {
				navController.navigate(Route.Appearance)
			},
			onPrivacyClick = {
				navController.navigate(Route.Privacy)
			},
			onNotificationsClick = {
				navController.navigate(Route.Notifications)
			},
			onBatterySettingsClick = {
				context.launchBatteryOptSettingsScreen()
			},
			onMixnetTuningClick = {
				navController.navigate(Route.MixnetTuning)
			},
			onGeoExclusionClick = {
				navController.navigate(Route.GeoExclusion)
			},
		),
	)
}

@Composable
fun SettingsScreen(values: SettingsValues, actions: SettingsActions) {
	Box(modifier = Modifier.fillMaxSize().background(MaterialTheme.colorScheme.background)) {
		Column(
			horizontalAlignment = Alignment.Start,
			verticalArrangement = Arrangement.spacedBy(14.dp, Alignment.Top),
			modifier = Modifier
				.verticalScroll(rememberScrollState())
				.fillMaxSize()
				.padding(top = 24.dp)
				.padding(horizontal = 16.dp.scaledWidth())
				.navigationBarsPadding(),
		) {
			LoginSection(
				isMnemonicStored = values.isMnemonicStored,
				onGetStartedClick = actions.onGetStartedClick,
			)
			AccountSection(
				isMnemonicStored = values.isMnemonicStored,
				onPassphraseClick = actions.onPassphraseClick,
				onAccountClick = actions.onAccountClick,
				subscription = values.subscription,
			)
			SupportSection(actions.onSupportClick)
			VpnSettingsSection(values, actions)
			AppearanceSection(values, actions)
			LogsSection(onPrivacyClick = actions.onPrivacyClick)
			// ResetAppSection(actions.onResetClick)
			LegalSection(actions.onLegalClick)
			// SystemStatusSection(actions.onSystemStatusClick)
			LogoutSection(isMnemonicStored = values.isMnemonicStored, actions.onLogoutClick)
			QuitSection(actions.onQuitClick)
			AppVersionSection(appVersion = values.appVersion, daemonVersion = values.daemonVersion, onAppVersionClick = actions.onAppVersionClick)
		}
	}
}

@Composable
@Preview(uiMode = Configuration.UI_MODE_NIGHT_YES)
internal fun PreviewSettingsScreen() {
	NymVPNTheme(Theme.default()) {
		SettingsScreen(
			SettingsValues(
				isMnemonicStored = true,
				subscription = SubscriptionUiState(
					isRecurring = false,
					validUntilDate = "December 24, 2026",
					expiryState = ExpiryState.NORMAL,
				),
			),
			SettingsActions(),
		)
	}
}
