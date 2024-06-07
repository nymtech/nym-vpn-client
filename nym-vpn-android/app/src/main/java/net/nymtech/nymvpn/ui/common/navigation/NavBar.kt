package net.nymtech.nymvpn.ui.common.navigation

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.foundation.layout.size
import androidx.compose.material3.CenterAlignedTopAppBar
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalSoftwareKeyboardController
import androidx.compose.ui.res.vectorResource
import androidx.navigation.NavController
import androidx.navigation.compose.currentBackStackEntryAsState
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.AppUiState
import net.nymtech.nymvpn.ui.NavItem
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.ui.theme.iconSize

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun NavBar(appUiState: AppUiState, navController: NavController, modifier: Modifier = Modifier) {
	val navBackStackEntry by navController.currentBackStackEntryAsState()
	val navItem = NavItem.from(navBackStackEntry?.destination?.route)
	val context = LocalContext.current
	val keyboardController = LocalSoftwareKeyboardController.current

	LaunchedEffect(navBackStackEntry) {
		keyboardController?.hide()
	}

	val emptyTitle = navItem.title.asString(context) == ""
	AnimatedVisibility(!emptyTitle, enter = fadeIn(), exit = fadeOut()) {
		CenterAlignedTopAppBar(
			modifier = modifier,
			title = {
				if (navItem.route == NavItem.Main.route) {
					val darkTheme =
						when (appUiState.settings.theme) {
							Theme.AUTOMATIC -> isSystemInDarkTheme()
							Theme.DARK_MODE -> true
							Theme.LIGHT_MODE -> false
							else -> true
						}
					if (darkTheme) {
						Icon(ImageVector.vectorResource(R.drawable.app_label_dark), "app_label", tint = Color.Unspecified)
					} else {
						Icon(ImageVector.vectorResource(R.drawable.app_label_light), "app_label", tint = Color.Unspecified)
					}
				} else {
					Text(
						navItem.title.asString(context),
						style = MaterialTheme.typography.titleLarge,
					)
				}
			},
			actions = {
				navItem.trailing?.let {
					IconButton(
						onClick = {
							when (it) {
								NavItem.settingsIcon -> navController.navigate(NavItem.Settings.route)
							}
						},
					) {
						Icon(
							imageVector = it,
							contentDescription = it.name,
							tint = MaterialTheme.colorScheme.onSurface,
							modifier =
							Modifier.size(
								iconSize,
							),
						)
					}
				}
			},
			navigationIcon = {
				navItem.leading?.let {
					IconButton(
						onClick = {
							when {
								it == NavItem.backIcon -> navController.popBackStack()
							}
						},
					) {
						Icon(imageVector = it, contentDescription = it.name)
					}
				}
			},
		)
	}
}
