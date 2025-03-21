package net.nymtech.nymvpn.ui.screens.settings.components

import android.content.Context
import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.outlined.ArrowRight
import androidx.compose.material.icons.outlined.AdminPanelSettings
import androidx.compose.material.icons.outlined.Lan
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.res.vectorResource
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.AppUiState
import net.nymtech.nymvpn.ui.common.buttons.ScaledSwitch
import net.nymtech.nymvpn.ui.common.buttons.surface.SelectionItem
import net.nymtech.nymvpn.ui.screens.settings.SettingsViewModel
import net.nymtech.nymvpn.ui.theme.iconSize
import net.nymtech.nymvpn.util.extensions.launchVpnSettings
import net.nymtech.nymvpn.util.extensions.scaledWidth

@Composable
fun VpnSettingsSection(appUiState: AppUiState, viewModel: SettingsViewModel, context: Context) {
	SettingsGroup(
		items = listOf(
			SelectionItem(
				leading = {
					Icon(
						ImageVector.vectorResource(R.drawable.auto),
						stringResource(R.string.auto_connect),
						modifier = Modifier.size(iconSize.scaledWidth()),
					)
				},
				trailing = {
					ScaledSwitch(
						checked = appUiState.settings.autoStartEnabled,
						onClick = { viewModel.onAutoConnectSelected(it) },
					)
				},
				title = {
					Text(
						stringResource(R.string.auto_connect),
						style = MaterialTheme.typography.bodyLarge.copy(MaterialTheme.colorScheme.onSurface),
					)
				},
				description = {
					Text(
						stringResource(R.string.auto_connect_description),
						style = MaterialTheme.typography.bodyMedium.copy(MaterialTheme.colorScheme.outline),
					)
				},
			),
			SelectionItem(
				leading = {
					Icon(
						Icons.Outlined.Lan,
						stringResource(R.string.bypass_lan),
						modifier = Modifier.size(iconSize.scaledWidth()),
					)
				},
				trailing = {
					ScaledSwitch(
						checked = appUiState.settings.isBypassLanEnabled,
						onClick = { viewModel.onBypassLanSelected(it) },
					)
				},
				title = {
					Text(
						stringResource(R.string.bypass_lan),
						style = MaterialTheme.typography.bodyLarge.copy(MaterialTheme.colorScheme.onSurface),
					)
				},
			),
			SelectionItem(
				leading = {
					Icon(
						Icons.Outlined.AdminPanelSettings,
						stringResource(R.string.kill_switch),
						modifier = Modifier.size(iconSize.scaledWidth()),
					)
				},
				trailing = {
					Icon(
						Icons.AutoMirrored.Outlined.ArrowRight,
						stringResource(R.string.go),
						modifier = Modifier.size(iconSize),
					)
				},
				title = {
					Text(
						stringResource(R.string.kill_switch),
						style = MaterialTheme.typography.bodyLarge.copy(MaterialTheme.colorScheme.onSurface),
					)
				},
				onClick = { context.launchVpnSettings() },
			),
		),
	)
}
