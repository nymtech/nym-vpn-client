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
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.rounded.Search
import androidx.compose.material3.Icon
import androidx.compose.material3.Text
import net.nymtech.nymvpn.ui.screens.settings.components.SettingsGroup
import net.nymtech.nymvpn.ui.screens.settings.components.accountEntries
import net.nymtech.nymvpn.ui.screens.settings.components.appEntries
import net.nymtech.nymvpn.ui.screens.settings.components.legalEntries
import net.nymtech.nymvpn.ui.screens.settings.components.matchesSearch
import net.nymtech.nymvpn.ui.screens.settings.components.logsEntries
import net.nymtech.nymvpn.ui.screens.settings.components.supportEntries
import net.nymtech.nymvpn.ui.screens.settings.components.vpnSettingsEntries
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import kotlinx.coroutines.flow.collectLatest
import net.nymtech.nymvpn.BuildConfig
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.theme.iconSize
import net.nymtech.nymvpn.ui.common.textbox.CustomTextField
import net.nymtech.nymvpn.ui.common.labels.GroupLabel
import net.nymtech.nymvpn.ui.AppUiState
import net.nymtech.nymvpn.ui.AppViewModel
import net.nymtech.nymvpn.ui.Route
import net.nymtech.nymvpn.ui.AuthRoute
import net.nymtech.nymvpn.ui.routeName
import net.nymtech.nymvpn.ui.common.events.UiEvent
import net.nymtech.nymvpn.ui.common.deviceauth.rememberScreenLockGate
import net.nymtech.nymvpn.ui.common.navigation.LocalNavController
import net.nymtech.nymvpn.ui.common.snackbar.AlertController
import net.nymtech.nymvpn.ui.screens.scan.RedeemCodeSheet
import net.nymtech.nymvpn.ui.screens.settings.components.AppVersionSection
import net.nymtech.nymvpn.ui.screens.settings.components.ExpiryState
import net.nymtech.nymvpn.ui.screens.settings.components.LoginSection
import net.nymtech.nymvpn.ui.screens.settings.components.LogoutDialog
import net.nymtech.nymvpn.ui.screens.settings.components.LogoutSection
import net.nymtech.nymvpn.ui.screens.settings.components.QuitSection
import net.nymtech.nymvpn.ui.screens.settings.components.SubscriptionUiState
import net.nymtech.nymvpn.ui.screens.settings.modal.PrivateDnsDialog
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.util.extensions.goFromRoot
import net.nymtech.nymvpn.util.extensions.isPrivateDnsEnabled
import net.nymtech.nymvpn.util.extensions.launchBatteryOptSettingsScreen
import net.nymtech.nymvpn.util.extensions.launchPrivateDnsSettings
import net.nymtech.nymvpn.util.extensions.launchVpnSettings
import androidx.compose.foundation.layout.navigationBarsPadding
import net.nymtech.nymvpn.util.extensions.scaledWidth
import kotlin.Boolean
import net.nymtech.qrscanner.isQrCameraAvailable

@Composable
fun SettingsScreen(appUiState: AppUiState, appViewModel: AppViewModel, showVpnSettings: Boolean = false, viewModel: SettingsViewModel = hiltViewModel()) {
	val context = LocalContext.current
	val navController = LocalNavController.current

	var loggingOut by remember { mutableStateOf(false) }
	var showLogoutDialog by remember { mutableStateOf(false) }
	var showPrivateDnsDialog by remember { mutableStateOf(false) }
	var showRedeemCodeSheet by rememberSaveable { mutableStateOf(false) }
	val cameraAvailable = remember(context) { isQrCameraAvailable(context) }

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
		onShowPassphrase = {
			showLogoutDialog = false
			navController.navigate(Route.Passphrase)
		},
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

	if (showRedeemCodeSheet) {
		RedeemCodeSheet(accountStored = appUiState.managerState.isMnemonicStored, onDismiss = { showRedeemCodeSheet = false })
	}

	val shortcutsLockGate = rememberScreenLockGate(
		setupBody = R.string.screen_lock_setup_shortcuts_body,
		onLockSetAfterSetup = { viewModel.onAppShortcutsSelected(true) },
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
			autoselectServerEnabled = false,
			appShortcutsEnabled = appUiState.settings.isShortcutsEnabled,
			appDeviceStartupEnabled = false,
			appVersion = BuildConfig.VERSION_NAME,
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
			onRedeemCodeClick = {
				if (cameraAvailable) navController.navigate(Route.QrScan()) else showRedeemCodeSheet = true
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
			onAutoselectServerEnable = {
			},
			onShortcutsEnable = { enable ->
				if (!enable || shortcutsLockGate.isOpen()) viewModel.onAppShortcutsSelected(enable)
			},
			onDeviceStartupEnable = {
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
			onProfilesClick = {
				navController.navigate(Route.Profiles)
			},
		),
	)
}

private val settingsSectionOrder = listOf(
	R.string.settings_section_account,
	R.string.settings_section_protection,
	R.string.settings_section_connection,
	R.string.settings_section_routing,
	R.string.settings_section_mixnet,
	R.string.settings_section_app,
	R.string.settings_section_help_data,
)

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
			var query by remember { mutableStateOf("") }
			val entries = accountEntries(
				isMnemonicStored = values.isMnemonicStored,
				onPassphraseClick = actions.onPassphraseClick,
				onAccountClick = actions.onAccountClick,
				onRedeemCodeClick = actions.onRedeemCodeClick,
				subscription = values.subscription,
			) + vpnSettingsEntries(values, actions) + appEntries(values, actions) +
				supportEntries(actions.onSupportClick) + logsEntries(actions.onPrivacyClick) + legalEntries(actions.onLegalClick)
			CustomTextField(
				value = query,
				onValueChange = { query = it },
				modifier = Modifier.fillMaxWidth(),
				placeholder = {
					Text(
						stringResource(R.string.settings_search_placeholder),
						color = MaterialTheme.colorScheme.onBackground,
					)
				},
				singleLine = true,
				leading = {
					Icon(
						Icons.Rounded.Search,
						contentDescription = stringResource(R.string.search),
						modifier = Modifier.size(iconSize),
					)
				},
				label = {},
				showClearIcon = true,
				textStyle = MaterialTheme.typography.bodyLarge.copy(color = MaterialTheme.colorScheme.onBackground),
				containerColor = MaterialTheme.colorScheme.background,
			)
			// while searching, keep only matching rows, each under its own section header
			val search = query.trim()
			val accountMatches = search.isEmpty() || matchesSearch(stringResource(R.string.settings_section_account), search)
			var anyShown = false
			settingsSectionOrder.forEach { section ->
				val isAccount = section == R.string.settings_section_account
				val items = entries.filter { entry ->
					entry.section == section &&
						(search.isEmpty() || (isAccount && accountMatches) || matchesSearch(entry.title, search))
				}.map { it.item }
				val showLogin = isAccount && accountMatches && !values.isMnemonicStored
				if (items.isNotEmpty() || showLogin) {
					anyShown = true
					GroupLabel(stringResource(section))
				}
				if (showLogin) {
					LoginSection(
						isMnemonicStored = values.isMnemonicStored,
						onGetStartedClick = actions.onGetStartedClick,
					)
				}
				if (items.isNotEmpty()) SettingsGroup(items = items)
			}
			if (!anyShown) {
				Text(
					stringResource(R.string.no_results_found),
					style = MaterialTheme.typography.bodyMedium,
					color = MaterialTheme.colorScheme.onBackground,
				)
			}
			LogoutSection(isMnemonicStored = values.isMnemonicStored, actions.onLogoutClick)
			QuitSection(actions.onQuitClick)
			AppVersionSection(appVersion = values.appVersion, onAppVersionClick = actions.onAppVersionClick)
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
