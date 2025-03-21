package net.nymtech.nymvpn.ui.screens.settings.components

import android.content.Context
import android.os.Build
import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.outlined.ArrowRight
import androidx.compose.material.icons.automirrored.outlined.ViewQuilt
import androidx.compose.material.icons.outlined.AppShortcut
import androidx.compose.material.icons.outlined.Notifications
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.AppUiState
import net.nymtech.nymvpn.ui.Route
import net.nymtech.nymvpn.ui.common.buttons.ScaledSwitch
import net.nymtech.nymvpn.ui.common.buttons.surface.SelectionItem
import net.nymtech.nymvpn.ui.common.navigation.LocalNavController
import net.nymtech.nymvpn.ui.screens.settings.SettingsViewModel
import net.nymtech.nymvpn.ui.theme.iconSize
import net.nymtech.nymvpn.util.extensions.launchNotificationSettings
import net.nymtech.nymvpn.util.extensions.scaledWidth

@Composable
fun AppearanceSection(appUiState: AppUiState, viewModel: SettingsViewModel, context: Context) {
	val navController = LocalNavController.current
	val items = mutableListOf(
		SelectionItem(
			leading = {
				Icon(
					Icons.AutoMirrored.Outlined.ViewQuilt,
					stringResource(R.string.appearance),
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
					stringResource(R.string.appearance),
					style = MaterialTheme.typography.bodyLarge.copy(MaterialTheme.colorScheme.onSurface),
				)
			},
			onClick = { navController.navigate(Route.Appearance) },
		),
		SelectionItem(
			leading = {
				Icon(
					Icons.Outlined.Notifications,
					stringResource(R.string.notifications),
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
					stringResource(R.string.notifications),
					style = MaterialTheme.typography.bodyLarge.copy(MaterialTheme.colorScheme.onSurface),
				)
			},
			onClick = { context.launchNotificationSettings() },
		),
	).apply {
		if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.N_MR1) {
			add(
				SelectionItem(
					leading = {
						Icon(
							Icons.Outlined.AppShortcut,
							stringResource(R.string.app_shortcuts),
							modifier = Modifier.size(iconSize.scaledWidth()),
						)
					},
					trailing = {
						ScaledSwitch(
							checked = appUiState.settings.isShortcutsEnabled,
							onClick = { viewModel.onAppShortcutsSelected(it) },
						)
					},
					title = {
						Text(
							stringResource(R.string.app_shortcuts),
							style = MaterialTheme.typography.bodyLarge.copy(MaterialTheme.colorScheme.onSurface),
						)
					},
					description = {
						Text(
							stringResource(R.string.enable_shortcuts),
							style = MaterialTheme.typography.bodyMedium.copy(MaterialTheme.colorScheme.outline),
						)
					},
				),
			)
		}
	}
	SettingsGroup(items = items)
}
