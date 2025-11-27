package net.nymtech.nymvpn.ui.screens.settings.components

import android.content.res.Configuration
import android.os.Build
import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.outlined.ArrowRight
import androidx.compose.material.icons.outlined.Apps
import androidx.compose.material.icons.outlined.BatterySaver
import androidx.compose.material.icons.outlined.Notifications
import androidx.compose.material.icons.outlined.RocketLaunch
import androidx.compose.material.icons.outlined.ViewComfy
import androidx.compose.material.icons.rounded.ViewComfy
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.res.vectorResource
import androidx.compose.ui.tooling.preview.Preview
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.common.buttons.ScaledSwitch
import net.nymtech.nymvpn.ui.common.buttons.surface.SelectionItem
import net.nymtech.nymvpn.ui.screens.settings.SettingsActions
import net.nymtech.nymvpn.ui.screens.settings.SettingsValues
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.ui.theme.iconSize
import net.nymtech.nymvpn.util.extensions.scaledWidth

@Composable
fun AppearanceSection(values: SettingsValues, actions: SettingsActions) {
	SettingsGroup(
		items = buildList {
			add(
				SelectionItem(
					leading = {
						Icon(
							Icons.Outlined.RocketLaunch,
							stringResource(R.string.settings_device_startup_title),
							modifier = Modifier.size(iconSize.scaledWidth()),
							tint = MaterialTheme.colorScheme.outline,
						)
					},
					trailing = {
						ScaledSwitch(
							checked = values.appDeviceStartupEnabled,
							onClick = actions.onDeviceStartupEnable,
						)
					},
					title = {
						Text(
							stringResource(R.string.settings_device_startup_title),
							style = MaterialTheme.typography.bodyLarge.copy(MaterialTheme.colorScheme.onSurface),
						)
					},
					description = {
						Text(
							stringResource(R.string.settings_device_startup_description),
							style = MaterialTheme.typography.bodySmall.copy(MaterialTheme.colorScheme.outline),
						)
					},
				),
			)
			add(
				SelectionItem(
					leading = {
						Icon(
							ImageVector.vectorResource(R.drawable.ic_system_tray),
							stringResource(R.string.settings_system_tray_title),
							modifier = Modifier.size(iconSize.scaledWidth()),
							tint = MaterialTheme.colorScheme.outline,
						)
					},
					trailing = {
						ScaledSwitch(
							checked = values.appSystemTrayEnabled,
							onClick = actions.onSystemTrayEnable,
						)
					},
					title = {
						Text(
							stringResource(R.string.settings_system_tray_title),
							style = MaterialTheme.typography.bodyLarge.copy(MaterialTheme.colorScheme.onSurface),
						)
					},
					description = {
						Text(
							stringResource(R.string.settings_system_tray_description),
							style = MaterialTheme.typography.bodySmall.copy(MaterialTheme.colorScheme.outline),
						)
					},
				),
			)
			add(
				SelectionItem(
					leading = {
						Icon(
							Icons.Outlined.ViewComfy,
							stringResource(R.string.appearance),
							modifier = Modifier.size(iconSize.scaledWidth()),
							tint = MaterialTheme.colorScheme.outline,
						)
					},
					trailing = {
						Icon(
							Icons.AutoMirrored.Outlined.ArrowRight,
							stringResource(R.string.go),
							modifier = Modifier.size(iconSize),
							tint = MaterialTheme.colorScheme.onBackground,
						)
					},
					title = {
						Text(
							stringResource(R.string.appearance),
							style = MaterialTheme.typography.bodyLarge.copy(MaterialTheme.colorScheme.onSurface),
						)
					},
					onClick = actions.onAppearanceClick,
				),
			)
			add(
				SelectionItem(
					leading = {
						Icon(
							Icons.Outlined.Notifications,
							stringResource(R.string.notifications),
							modifier = Modifier.size(iconSize.scaledWidth()),
							tint = MaterialTheme.colorScheme.outline,
						)
					},
					trailing = {
						Icon(
							Icons.AutoMirrored.Outlined.ArrowRight,
							stringResource(R.string.go),
							modifier = Modifier.size(iconSize),
							tint = MaterialTheme.colorScheme.onBackground,
						)
					},
					title = {
						Text(
							stringResource(R.string.notifications),
							style = MaterialTheme.typography.bodyLarge.copy(MaterialTheme.colorScheme.onSurface),
						)
					},
					onClick = actions.onNotificationsClick,
				),
			)
			add(
				SelectionItem(
					leading = {
						Icon(
							Icons.Outlined.BatterySaver,
							stringResource(R.string.battery_opt),
							modifier = Modifier.size(iconSize.scaledWidth()),
							tint = MaterialTheme.colorScheme.outline,
						)
					},
					trailing = {
						Icon(
							Icons.AutoMirrored.Outlined.ArrowRight,
							stringResource(R.string.go),
							modifier = Modifier.size(iconSize),
							tint = MaterialTheme.colorScheme.onBackground,
						)
					},
					title = {
						Text(
							stringResource(R.string.battery_opt),
							style = MaterialTheme.typography.bodyLarge.copy(MaterialTheme.colorScheme.onSurface),
						)
					},
					onClick = actions.onBatterySettingsClick,
				),
			)
			if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.N_MR1) {
				add(
					SelectionItem(
						leading = {
							Icon(
								Icons.Outlined.Apps,
								stringResource(R.string.app_shortcuts),
								modifier = Modifier.size(iconSize.scaledWidth()),
								tint = MaterialTheme.colorScheme.outline,
							)
						},
						trailing = {
							ScaledSwitch(
								checked = values.appShortcutsEnabled,
								onClick = actions.onShortcutsEnable,
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
		},
	)
}

@Composable
@Preview(uiMode = Configuration.UI_MODE_NIGHT_YES)
internal fun PreviewAppearanceSections() {
	NymVPNTheme(Theme.default()) {
		AppearanceSection(
			SettingsValues(),
			SettingsActions(),
		)
	}
}
