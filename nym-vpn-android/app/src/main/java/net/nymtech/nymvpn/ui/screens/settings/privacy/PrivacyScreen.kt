package net.nymtech.nymvpn.ui.screens.settings.privacy

import android.content.res.Configuration
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.Notes
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.AppUiState
import net.nymtech.nymvpn.ui.Route
import net.nymtech.nymvpn.ui.common.TelemetryInfoKind
import net.nymtech.nymvpn.ui.common.WhatsSharedDialog
import net.nymtech.nymvpn.ui.common.buttons.surface.SelectionItem
import net.nymtech.nymvpn.ui.common.navigation.LocalNavController
import net.nymtech.nymvpn.ui.screens.settings.components.SettingsArrowIcon
import net.nymtech.nymvpn.ui.screens.settings.components.SettingsGroup
import net.nymtech.nymvpn.ui.screens.settings.components.SettingsIcon
import net.nymtech.nymvpn.ui.screens.settings.components.SettingsTitle
import net.nymtech.nymvpn.ui.screens.settings.diagnostic.components.RunDiagnosticsButton
import net.nymtech.nymvpn.ui.screens.settings.privacy.components.LoggingLevel
import net.nymtech.nymvpn.ui.screens.settings.privacy.components.LoggingLevelCard
import net.nymtech.nymvpn.ui.screens.settings.privacy.components.MonitoringSection
import net.nymtech.nymvpn.ui.screens.settings.privacy.components.NetworkStatsSection
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.util.extensions.scaledHeight
import net.nymtech.nymvpn.util.extensions.scaledWidth

@Composable
fun PrivacyScreen(appUiState: AppUiState, viewModel: PrivacyViewModel = hiltViewModel()) {
	val navController = LocalNavController.current
	val loggingLevel = LoggingLevel.from(appUiState.settings.logsEnabled, appUiState.vpnConfig.debugLog)
	PrivacyScreen(
		choices = PrivacyChoices(
			statsEnabled = appUiState.settings.statsEnabled,
			sentryEnabled = appUiState.vpnConfig.sentry,
			loggingLevel = loggingLevel,
			loggingRestartPending = viewModel.isRestartPending(loggingLevel),
		),
		onNetworkStatsEnable = {
			viewModel.onNetworkStatsEnabled(it)
		},
		onMonitoringEnable = {
			viewModel.onMonitoringEnabled(it)
		},
		onLoggingLevelChange = {
			viewModel.onLoggingLevelChange(it)
		},
		onLogsClick = {
			navController.navigate(Route.Logs)
		},
		onDiagnosticClick = {
			navController.navigate(Route.Diagnostic)
		},
	)
}

@Composable
fun PrivacyScreen(
	choices: PrivacyChoices,
	onNetworkStatsEnable: (enabled: Boolean) -> Unit,
	onMonitoringEnable: (enabled: Boolean) -> Unit,
	onLoggingLevelChange: (LoggingLevel) -> Unit,
	onLogsClick: () -> Unit,
	onDiagnosticClick: () -> Unit,
) {
	var infoKind by remember { mutableStateOf<TelemetryInfoKind?>(null) }
	Column(
		horizontalAlignment = Alignment.Start,
		verticalArrangement = Arrangement.spacedBy(14.dp.scaledHeight(), Alignment.Top),
		modifier = Modifier
			.fillMaxSize()
			.verticalScroll(rememberScrollState())
			.padding(top = 24.dp.scaledHeight())
			.padding(horizontal = 24.dp.scaledWidth()),
	) {
		NetworkStatsSection(choices.statsEnabled, onNetworkStatsEnable, onInfo = { infoKind = it })
		MonitoringSection(choices.sentryEnabled, onMonitoringEnable, onInfo = { infoKind = it })
		LoggingLevelCard(level = choices.loggingLevel, showRestartNote = choices.loggingRestartPending, onLevelChange = onLoggingLevelChange)

		SettingsGroup(
			items = listOf(
				SelectionItem(
					leading = {
						SettingsIcon(
							Icons.AutoMirrored.Default.Notes,
							stringResource(R.string.local_logs),
						)
					},
					trailing = {
						Row(
							verticalAlignment = Alignment.CenterVertically,
							horizontalArrangement = Arrangement.spacedBy(4.dp),
						) {
							Text(
								text = stringResource(choices.loggingLevel.labelRes),
								style = MaterialTheme.typography.bodyMedium,
								color = MaterialTheme.colorScheme.onBackground,
							)
							SettingsArrowIcon()
						}
					},
					title = {
						SettingsTitle(
							stringResource(R.string.local_logs),
						)
					},
					description = {
						Text(
							text = stringResource(R.string.local_logs_description),
							style = MaterialTheme.typography.bodySmall,
							color = MaterialTheme.colorScheme.onBackground,
						)
					},
					onClick = onLogsClick,
				),
			),
		)

		RunDiagnosticsButton(onClick = onDiagnosticClick)
	}

	infoKind?.let { kind ->
		WhatsSharedDialog(kind = kind, onDismiss = { infoKind = null })
	}
}

@Composable
@Preview(uiMode = Configuration.UI_MODE_NIGHT_YES)
internal fun PreviewPrivacyScreen() {
	NymVPNTheme(Theme.default()) {
		PrivacyScreen(
			choices = PrivacyChoices(statsEnabled = true, sentryEnabled = true, loggingLevel = LoggingLevel.BASIC, loggingRestartPending = false),
			onNetworkStatsEnable = {},
			onMonitoringEnable = {},
			onLoggingLevelChange = {},
			onLogsClick = {},
			onDiagnosticClick = {},
		)
	}
}
