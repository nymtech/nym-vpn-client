package net.nymtech.nymvpn.ui.screens.settings.appearance.components

import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.outlined.ArrowRight
import androidx.compose.material.icons.outlined.Contrast
import androidx.compose.material.icons.outlined.Translate
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.navigation.NavController
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.Route
import net.nymtech.nymvpn.ui.common.buttons.surface.SelectionItem
import net.nymtech.nymvpn.ui.screens.settings.components.SettingsGroup
import net.nymtech.nymvpn.ui.theme.iconSize
import net.nymtech.nymvpn.util.extensions.scaledWidth

@Composable
fun AppearanceOptions(navController: NavController) {
	SettingsGroup(
		items = listOf(
			SelectionItem(
				leading = {
					Icon(
						Icons.Outlined.Translate,
						stringResource(R.string.language),
						modifier = Modifier.size(iconSize.scaledWidth()),
					)
				},
				trailing = {
					Icon(
						Icons.AutoMirrored.Outlined.ArrowRight,
						stringResource(R.string.go),
						Modifier.size(iconSize),
					)
				},
				title = {
					Text(
						stringResource(R.string.language),
						style = MaterialTheme.typography.bodyLarge.copy(MaterialTheme.colorScheme.onSurface),
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
					Icon(
						Icons.Outlined.Contrast,
						stringResource(R.string.display_theme),
						modifier = Modifier.size(iconSize.scaledWidth()),
					)
				},
				trailing = {
					Icon(
						Icons.AutoMirrored.Outlined.ArrowRight,
						stringResource(R.string.go),
						Modifier.size(iconSize),
					)
				},
				title = {
					Text(
						stringResource(R.string.display_theme),
						style = MaterialTheme.typography.bodyLarge.copy(MaterialTheme.colorScheme.onSurface),
					)
				},
				onClick = { navController.navigate(Route.Display) },
			),
		),
	)
}
