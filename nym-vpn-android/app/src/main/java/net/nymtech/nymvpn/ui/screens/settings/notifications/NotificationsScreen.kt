package net.nymtech.nymvpn.ui.screens.settings.notifications

import android.content.res.Configuration
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Groups
import androidx.compose.material.icons.outlined.WebAsset
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.AppUiState
import net.nymtech.nymvpn.ui.common.buttons.ScaledSwitch
import net.nymtech.nymvpn.ui.common.buttons.surface.SelectionItem
import net.nymtech.nymvpn.ui.screens.settings.components.SettingsArrowIcon
import net.nymtech.nymvpn.ui.screens.settings.components.SettingsGroup
import net.nymtech.nymvpn.ui.screens.settings.components.SettingsIcon
import net.nymtech.nymvpn.ui.screens.settings.components.SettingsTitle
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.util.extensions.launchNotificationSettings
import net.nymtech.nymvpn.util.extensions.scaledHeight
import net.nymtech.nymvpn.util.extensions.scaledWidth

@Composable
fun NotificationsScreen(appUiState: AppUiState, viewModel: NotificationsViewModel = hiltViewModel()) {
	val context = LocalContext.current
	NotificationsScreen(
		nodeFamiliesEnabled = appUiState.vpnConfig.nodeFamiliesNotificationsEnabled,
		onNodeFamiliesEnable = { viewModel.onNodeFamiliesEnabled(it) },
		onSystemNotificationsClick = {
			context.launchNotificationSettings()
		},
	)
}

@Composable
fun NotificationsScreen(nodeFamiliesEnabled: Boolean, onNodeFamiliesEnable: (enabled: Boolean) -> Unit, onSystemNotificationsClick: () -> Unit) {
	Column(
		horizontalAlignment = Alignment.Start,
		verticalArrangement = Arrangement.spacedBy(14.dp.scaledHeight(), Alignment.Top),
		modifier = Modifier
			.fillMaxSize()
			.padding(top = 24.dp.scaledHeight())
			.padding(horizontal = 24.dp.scaledWidth()),
	) {
		SettingsGroup(
			items = listOf(
				SelectionItem(
					leading = {
						SettingsIcon(
							Icons.Filled.Groups,
							stringResource(R.string.notifications_node_families_title),
						)
					},

					trailing = {
						ScaledSwitch(
							checked = nodeFamiliesEnabled,
							onClick = { onNodeFamiliesEnable(it) },
						)
					},
					title = {
						SettingsTitle(
							stringResource(R.string.notifications_node_families_title),
						)
					},
					description = {
						Text(
							stringResource(R.string.notifications_node_families_description),
							style = MaterialTheme.typography.bodySmall,
							color = MaterialTheme.colorScheme.onBackground,
						)
					},
				),
				SelectionItem(
					leading = {
						SettingsIcon(
							Icons.Outlined.WebAsset,
							stringResource(R.string.notifications_system_title),
						)
					},
					trailing = {
						SettingsArrowIcon()
					},
					title = {
						SettingsTitle(
							stringResource(R.string.notifications_system_title),
						)
					},
					description = {
						Text(
							stringResource(R.string.notifications_system_description),
							style = MaterialTheme.typography.bodySmall,
							color = MaterialTheme.colorScheme.onBackground,
						)
					},
					onClick = onSystemNotificationsClick,
				),
			),
		)
	}
}

@Composable
@Preview(uiMode = Configuration.UI_MODE_NIGHT_YES)
internal fun PreviewNotificationsScreen() {
	NymVPNTheme(Theme.default()) {
		NotificationsScreen(
			nodeFamiliesEnabled = true,
			onNodeFamiliesEnable = {
			},
			onSystemNotificationsClick = {
			},
		)
	}
}
