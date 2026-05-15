package net.nymtech.nymvpn.ui.screens.technical

import android.content.res.Configuration
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.asPaddingValues
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.systemBars
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.AppUiState
import net.nymtech.nymvpn.ui.Route
import net.nymtech.nymvpn.ui.common.buttons.MainStyledButton
import net.nymtech.nymvpn.ui.common.navigation.LocalNavController
import net.nymtech.nymvpn.ui.screens.technical.components.SettingsSection
import net.nymtech.nymvpn.ui.screens.technical.components.WelcomeSection
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.util.extensions.replaceCurrentWith
import net.nymtech.nymvpn.util.extensions.scaledHeight
import net.nymtech.nymvpn.util.extensions.scaledWidth

@Composable
fun TechnicalOptScreen(appUiState: AppUiState, viewModel: TechnicalOptViewModel = hiltViewModel()) {
	val navController = LocalNavController.current
	TechnicalOptScreen(
		statsEnabled = appUiState.settings.statsEnabled,
		sentryEnabled = appUiState.vpnConfig.sentry,
		onNetworkStatsEnable = {
			viewModel.onNetworkStatsEnabled(it)
		},
		onMonitoringEnable = {
			viewModel.onMonitoringEnabled(it)
		},
		onContinueClick = {
			viewModel.onContinueClicked()
			navController.replaceCurrentWith(Route.Main())
		},
	)
}

@Composable
fun TechnicalOptScreen(
	statsEnabled: Boolean,
	sentryEnabled: Boolean,
	onNetworkStatsEnable: (enabled: Boolean) -> Unit,
	onMonitoringEnable: (enabled: Boolean) -> Unit,
	onContinueClick: () -> Unit,
	padding: PaddingValues = WindowInsets.systemBars.asPaddingValues(),
) {
	Column(
		horizontalAlignment = Alignment.CenterHorizontally,
		modifier = Modifier
			.fillMaxSize()
			.imePadding()
			.padding(horizontal = 24.dp.scaledWidth())
			.padding(padding),
	) {
		WelcomeSection(modifier = Modifier.padding(vertical = 24.dp.scaledHeight()).weight(1f))
		Column(
			horizontalAlignment = Alignment.CenterHorizontally,
			verticalArrangement = Arrangement.spacedBy(16.dp, Alignment.Bottom),
			modifier = Modifier.padding(vertical = 24.dp.scaledHeight()),
		) {
			SettingsSection(statsEnabled, sentryEnabled, onNetworkStatsEnable, onMonitoringEnable)
			Column(
				horizontalAlignment = Alignment.CenterHorizontally,
				modifier = Modifier.padding(top = 8.dp),
			) {
				MainStyledButton(
					onClick = onContinueClick,
					content = {
						Text(
							text = stringResource(R.string.welcome_continue),
							style = MaterialTheme.typography.titleMedium,
						)
					},
					color = MaterialTheme.colorScheme.primary,
					modifier = Modifier
						.fillMaxWidth()
						.height(48.dp.scaledHeight()),
					shape = RoundedCornerShape(12.dp),
				)
			}
		}
	}
}

@Composable
@Preview(uiMode = Configuration.UI_MODE_NIGHT_YES)
internal fun PreviewWelcomeScreen() {
	NymVPNTheme(Theme.default()) {
		TechnicalOptScreen(
			statsEnabled = true,
			sentryEnabled = true,
			onMonitoringEnable = {},
			onNetworkStatsEnable = {},
			onContinueClick = {},
		)
	}
}
