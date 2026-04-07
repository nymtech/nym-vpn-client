package net.nymtech.nymvpn.ui.screens.settings

import android.app.Activity
import android.content.res.Configuration
import android.widget.Toast
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
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
import net.nymtech.nymvpn.ui.Route
import net.nymtech.nymvpn.ui.common.events.UiEvent
import net.nymtech.nymvpn.ui.common.navigation.LocalNavController
import net.nymtech.nymvpn.ui.screens.settings.components.AccountSection
import net.nymtech.nymvpn.ui.screens.settings.components.AppVersionSection
import net.nymtech.nymvpn.ui.screens.settings.components.AppearanceSection
import net.nymtech.nymvpn.ui.screens.settings.components.ExpiryState
import net.nymtech.nymvpn.ui.screens.settings.components.LegalSection
import net.nymtech.nymvpn.ui.screens.settings.components.LoginSection
import net.nymtech.nymvpn.ui.screens.settings.components.LogsSection
import net.nymtech.nymvpn.ui.screens.settings.components.QuitSection
import net.nymtech.nymvpn.ui.screens.settings.components.SubscriptionUiState
import net.nymtech.nymvpn.ui.screens.settings.components.SupportSection
import net.nymtech.nymvpn.ui.screens.settings.components.VpnSettingsSection
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.util.DeviceAuthHelper
import net.nymtech.nymvpn.util.extensions.launchBatteryOptSettingsScreen
import net.nymtech.nymvpn.util.extensions.launchNotificationSettings
import net.nymtech.nymvpn.util.extensions.launchVpnSettings
import net.nymtech.nymvpn.util.extensions.scaledWidth
import kotlin.Boolean

@Composable
fun SettingsScreen(appUiState: AppUiState, showVpnSettings: Boolean = false, viewModel: SettingsViewModel = hiltViewModel()) {
	val context = LocalContext.current
	val navController = LocalNavController.current

	val uiState by viewModel.uiState.collectAsStateWithLifecycle()

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
			supportIPv6Enabled = false,
			autoselectServerEnabled = false,
			appShortcutsEnabled = appUiState.settings.isShortcutsEnabled,
			appDeviceStartupEnabled = false,
			appSystemTrayEnabled = false,
			appVersion = BuildConfig.VERSION_NAME,
			daemonVersion = uiState.daemonVersion,
			isMixnetTuningEnabled = uiState.isMixnetTuningEnabled,
			subscription = appUiState.subscription,
			isLewesEnabled = appUiState.vpnConfig.lewes,
		),
		SettingsActions(
			onGetStartedClick = {
				navController.navigate(Route.Welcome)
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
			onAppVersionClick = {
				navController.navigate(Route.Developer)
			},
			onSplitTunnelingClick = {
				navController.navigate(Route.SplitTunneling)
			},
			onAutoConnectEnable = { viewModel.onAutoConnectSelected(it) },
			onBypassLanEnable = { viewModel.onBypassLanSelected(it) },
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
				context.launchNotificationSettings()
			},
			onBatterySettingsClick = {
				context.launchBatteryOptSettingsScreen()
			},
			onMixnetTuningClick = {
				navController.navigate(Route.MixnetTuning)
			},
			onLewesEnable = {
				viewModel.onLewesProtocolSelected(it)
			},
		),
	)
}

@Composable
fun SettingsScreen(values: SettingsValues, actions: SettingsActions) {
	Box(modifier = Modifier.fillMaxSize()) {
		Column(
			horizontalAlignment = Alignment.Start,
			verticalArrangement = Arrangement.spacedBy(24.dp, Alignment.Top),
			modifier = Modifier
				.verticalScroll(rememberScrollState())
				.fillMaxSize()
				.padding(top = 24.dp)
				.padding(horizontal = 24.dp.scaledWidth()),
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
