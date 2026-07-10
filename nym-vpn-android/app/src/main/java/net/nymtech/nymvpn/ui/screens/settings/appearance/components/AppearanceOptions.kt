package net.nymtech.nymvpn.ui.screens.settings.appearance.components

import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Contrast
import androidx.compose.material.icons.outlined.Palette
import androidx.compose.material.icons.outlined.Translate
import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import androidx.navigation.NavController
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.Route
import net.nymtech.nymvpn.ui.common.buttons.surface.SelectionItem
import net.nymtech.nymvpn.ui.screens.settings.components.SettingsArrowIcon
import net.nymtech.nymvpn.ui.screens.settings.components.SettingsGroup
import net.nymtech.nymvpn.ui.screens.settings.components.SettingsIcon
import net.nymtech.nymvpn.ui.screens.settings.components.SettingsTitle

@Composable
fun AppearanceOptions(navController: NavController) {
	SettingsGroup(
		items = listOf(
			SelectionItem(
				leading = {
					SettingsIcon(
						Icons.Outlined.Translate,
						stringResource(R.string.language),
					)
				},
				trailing = {
					SettingsArrowIcon()
				},
				title = {
					SettingsTitle(
						stringResource(R.string.language),
					)
				},
				onClick = { navController.navigate(Route.Language) },
			),
		),
	)
	SettingsGroup(
		items = listOf(
			SelectionItem(
				leading = {
					SettingsIcon(
						Icons.Outlined.Contrast,
						stringResource(R.string.display_theme),
					)
				},
				trailing = {
					SettingsArrowIcon()
				},
				title = {
					SettingsTitle(
						stringResource(R.string.display_theme),
					)
				},
				onClick = { navController.navigate(Route.Display) },
			),
		),
	)
	SettingsGroup(
		items = listOf(
			SelectionItem(
				leading = {
					SettingsIcon(
						Icons.Outlined.Palette,
						stringResource(R.string.app_icon_title),
					)
				},
				trailing = {
					SettingsArrowIcon()
				},
				title = {
					SettingsTitle(
						stringResource(R.string.app_icon_title),
					)
				},
				onClick = { navController.navigate(Route.AppIcon) },
			),
		),
	)
}
