package net.nymtech.nymvpn.ui.screens.settings.privacy

import android.content.res.Configuration
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import net.nymtech.nymvpn.ui.AppUiState
import net.nymtech.nymvpn.ui.screens.settings.privacy.components.MonitoringSection
import net.nymtech.nymvpn.ui.screens.settings.privacy.components.NetworkStatsSection
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.util.extensions.scaledHeight
import net.nymtech.nymvpn.util.extensions.scaledWidth

@Composable
fun PrivacyScreen(appUiState: AppUiState, viewModel: PrivacyViewModel = hiltViewModel()) {
	PrivacyScreen(
		appUiState.settings.statsEnabled,
		appUiState.vpnConfig.sentry,
		onNetworkStatsEnable = {
			viewModel.onNetworkStatsEnabled(it)
		},
		onMonitoringEnable = {
			viewModel.onMonitoringEnabled(it)
		},
	)
}

@Composable
fun PrivacyScreen(statsEnabled: Boolean, sentryEnabled: Boolean, onNetworkStatsEnable: (enabled: Boolean) -> Unit, onMonitoringEnable: (enabled: Boolean) -> Unit) {
	val context = LocalContext.current
	Column(
		horizontalAlignment = Alignment.Start,
		verticalArrangement = Arrangement.spacedBy(24.dp.scaledHeight(), Alignment.Top),
		modifier = Modifier
			.fillMaxSize()
			.padding(top = 24.dp.scaledHeight())
			.padding(horizontal = 24.dp.scaledWidth()),
	) {
		NetworkStatsSection(statsEnabled, onNetworkStatsEnable, context)
		MonitoringSection(sentryEnabled, onMonitoringEnable, context)
	}
}

@Composable
@Preview(uiMode = Configuration.UI_MODE_NIGHT_YES)
internal fun PreviewPrivacyScreen() {
	NymVPNTheme(Theme.default()) {
		PrivacyScreen(statsEnabled = true, sentryEnabled = true, onMonitoringEnable = {}, onNetworkStatsEnable = {})
	}
}
