package net.nymtech.nymvpn.ui.screens.settings.components

import android.content.res.Configuration
import android.os.Build
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.BatterySaver
import androidx.compose.material.icons.outlined.KeyboardAlt
import androidx.compose.material.icons.outlined.Notifications
import androidx.compose.material.icons.outlined.RocketLaunch
import androidx.compose.material.icons.outlined.ViewComfy
import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.common.buttons.ScaledSwitch
import net.nymtech.nymvpn.ui.common.buttons.surface.SelectionItem
import net.nymtech.nymvpn.ui.screens.settings.SettingsActions
import net.nymtech.nymvpn.ui.screens.settings.SettingsValues
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme

@Composable
fun AppearanceSection(values: SettingsValues, actions: SettingsActions) {
	SettingsGroup(
		items = buildList {
			add(
				SelectionItem(
					leading = {
						SettingsIcon(
							Icons.Outlined.RocketLaunch,
							stringResource(R.string.settings_startup_title),
						)
					},
					trailing = {
						ScaledSwitch(
							checked = values.autoConnectEnabled,
							onClick = actions.onAutoConnectEnable,
						)
					},
					title = {
						SettingsTitle(stringResource(R.string.settings_startup_title))
					},
				),
			)

			add(
				SelectionItem(
					leading = {
						SettingsIcon(
							Icons.Outlined.Notifications,
							stringResource(R.string.settings_notifications_title),
						)
					},
					trailing = {
						SettingsArrowIcon()
					},
					title = {
						SettingsTitle(stringResource(R.string.settings_notifications_title))
					},
					onClick = actions.onNotificationsClick,
				),
			)
			add(
				SelectionItem(
					leading = {
						SettingsIcon(
							Icons.Outlined.BatterySaver,
							stringResource(R.string.settings_power_managment_title),
						)
					},
					trailing = {
						SettingsArrowIcon()
					},
					title = {
						SettingsTitle(stringResource(R.string.settings_power_managment_title))
					},
					onClick = actions.onBatterySettingsClick,
				),
			)
			if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.N_MR1) {
				add(
					SelectionItem(
						leading = {
							SettingsIcon(
								Icons.Outlined.KeyboardAlt,
								stringResource(R.string.settings_shortcuts_title),
							)
						},
						trailing = {
							ScaledSwitch(
								checked = values.appShortcutsEnabled,
								onClick = actions.onShortcutsEnable,
							)
						},
						title = {
							SettingsTitle(stringResource(R.string.settings_shortcuts_title))
						},
					),
				)
			}

			add(
				SelectionItem(
					leading = {
						SettingsIcon(
							Icons.Outlined.ViewComfy,
							stringResource(R.string.appearance),
						)
					},
					trailing = {
						SettingsArrowIcon()
					},
					title = {
						SettingsTitle(stringResource(R.string.appearance))
					},
					onClick = actions.onAppearanceClick,
				),
			)
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
