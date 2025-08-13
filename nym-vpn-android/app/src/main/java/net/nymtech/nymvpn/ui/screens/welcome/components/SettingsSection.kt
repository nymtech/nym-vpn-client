package net.nymtech.nymvpn.ui.screens.welcome.components

import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.BarChart
import androidx.compose.material.icons.outlined.BugReport
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.AppUiState
import net.nymtech.nymvpn.ui.common.buttons.ScaledSwitch
import net.nymtech.nymvpn.ui.common.buttons.surface.SelectionItem
import net.nymtech.nymvpn.ui.screens.settings.components.SettingsGroup
import net.nymtech.nymvpn.ui.screens.welcome.WelcomeViewModel
import net.nymtech.nymvpn.ui.theme.iconSize
import net.nymtech.nymvpn.util.extensions.scaledWidth

@Composable
fun SettingsSection(appUiState: AppUiState, viewModel: WelcomeViewModel) {

	SettingsGroup(
		modifier = Modifier.padding(top = 12.dp, bottom = 12.dp),
		items = listOf(
			SelectionItem(
				leading = {
					Icon(
						Icons.Outlined.BarChart,
						stringResource(R.string.title_anonymous_stats),
						modifier = Modifier.size(iconSize.scaledWidth()),
					)
				},
				trailing = {
					ScaledSwitch(
						checked = appUiState.settings.statsEnabled,
						onClick = { viewModel.onNetworkStatsEnabled(it) },
					)
				},
				title = {
					Text(
						stringResource(R.string.title_anonymous_stats),
						style = MaterialTheme.typography.bodyLarge.copy(MaterialTheme.colorScheme.onSurface),
					)
				},
			),
			SelectionItem(
				leading = {
					Icon(
						Icons.Outlined.BugReport,
						stringResource(R.string.title_error_reports),
						modifier = Modifier.size(iconSize.scaledWidth()),
					)
				},
				trailing = {
					ScaledSwitch(
						checked = appUiState.settings.sentryEnabled,
						onClick = { viewModel.onMonitoringEnabled(it) },
					)
				},
				title = {
					Text(
						stringResource(R.string.title_error_reports),
						style = MaterialTheme.typography.bodyLarge.copy(MaterialTheme.colorScheme.onSurface),
					)
				},
				description = {
					Text(
						stringResource(R.string.welcome_error_toggle_descr),
						style = MaterialTheme.typography.bodyMedium.copy(MaterialTheme.colorScheme.outline),
					)
				},
			),
		),
	)
}
