package net.nymtech.nymvpn.ui.screens.settings.appearance

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.automirrored.outlined.ArrowRight
import androidx.compose.material.icons.outlined.Contrast
import androidx.compose.material.icons.outlined.Translate
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.AppViewModel
import net.nymtech.nymvpn.ui.Route
import net.nymtech.nymvpn.ui.common.buttons.surface.SelectionItem
import net.nymtech.nymvpn.ui.common.buttons.surface.SurfaceSelectionGroupButton
import net.nymtech.nymvpn.ui.common.navigation.LocalNavController
import net.nymtech.nymvpn.ui.common.navigation.NavBarState
import net.nymtech.nymvpn.ui.common.navigation.NavIcon
import net.nymtech.nymvpn.ui.common.navigation.NavTitle
import net.nymtech.nymvpn.ui.theme.iconSize
import net.nymtech.nymvpn.util.extensions.scaledHeight
import net.nymtech.nymvpn.util.extensions.scaledWidth

@Composable
fun AppearanceScreen(appViewModel: AppViewModel) {
	val navController = LocalNavController.current
	LaunchedEffect(Unit) {
		appViewModel.onNavBarStateChange(
			NavBarState(
				title = { NavTitle(stringResource(R.string.appearance)) },
				leading = {
					NavIcon(Icons.AutoMirrored.Filled.ArrowBack, stringResource(R.string.back)) {
						navController.popBackStack()
					}
				},
			),
		)
	}

	Column(
		horizontalAlignment = Alignment.Start,
		verticalArrangement = Arrangement.spacedBy(24.dp.scaledHeight(), Alignment.Top),
		modifier =
		Modifier
			.fillMaxSize()
			.padding(top = 24.dp.scaledHeight())
			.padding(horizontal = 24.dp.scaledWidth()),
	) {
		SurfaceSelectionGroupButton(
			listOf(
				SelectionItem(
					{
						val icon = Icons.Outlined.Translate
						Icon(
							icon,
							stringResource(R.string.language),
							modifier = Modifier.size(iconSize.scaledWidth()),
						)
					},
					{
						val icon = Icons.AutoMirrored.Outlined.ArrowRight
						Icon(icon, stringResource(R.string.go), Modifier.size(iconSize))
					},
					title = { Text(stringResource(R.string.language), style = MaterialTheme.typography.bodyLarge.copy(MaterialTheme.colorScheme.onSurface)) },
					onClick = { navController.navigate(Route.Language) },
				),
			),
			background = MaterialTheme.colorScheme.surface,
		)
		SurfaceSelectionGroupButton(
			listOf(
				SelectionItem(
					{
						val icon = Icons.Outlined.Contrast
						Icon(
							icon,
							stringResource(R.string.display_theme),
							modifier = Modifier.size(iconSize.scaledWidth()),
						)
					},
					{
						val icon = Icons.AutoMirrored.Outlined.ArrowRight
						Icon(icon, stringResource(R.string.go), Modifier.size(iconSize))
					},
					title = { Text(stringResource(R.string.display_theme), style = MaterialTheme.typography.bodyLarge.copy(MaterialTheme.colorScheme.onSurface)) },
					onClick = { navController.navigate(Route.Display) },
				),
			),
			background = MaterialTheme.colorScheme.surface,
		)
	}
}
