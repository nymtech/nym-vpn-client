package net.nymtech.nymvpn.ui.common.navigation

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.slideInVertically
import androidx.compose.animation.slideOutVertically
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.outlined.Info
import androidx.compose.material.icons.outlined.Settings
import androidx.compose.material3.CenterAlignedTopAppBar
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.TopAppBarDefaults
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalSoftwareKeyboardController
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.navigation.NavController
import androidx.navigation.compose.currentBackStackEntryAsState
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.Route
import net.nymtech.nymvpn.ui.common.Modal
import net.nymtech.nymvpn.ui.screens.hop.components.ServerDetailsModalBody
import net.nymtech.nymvpn.ui.theme.CustomTypography
import net.nymtech.nymvpn.util.extensions.goFromRoot
import net.nymtech.nymvpn.util.extensions.openWebUrl

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun NavBar(navController: NavController, modifier: Modifier = Modifier) {
	val keyboardController = LocalSoftwareKeyboardController.current
	val context = LocalContext.current

	val navBackStackEntry by navController.currentBackStackEntryAsState()
	var navBarState by remember { mutableStateOf(NavBarState()) }
	var showLocationTooltip by remember { mutableStateOf(false) }

	LaunchedEffect(navBackStackEntry) {
		keyboardController?.hide()
		val currentRoute = navBackStackEntry?.destination?.route ?: return@LaunchedEffect
		navBarState = when {
			currentRoute.startsWith(Route.Splash::class.qualifiedName!!) -> NavBarState(
				show = false,
			)
			currentRoute.startsWith(Route.Main::class.qualifiedName!!) -> {
				NavBarState(
					title = { MainTitle() },
					show = true,
					trailing = {
						NavIcon(Icons.Outlined.Settings, stringResource(R.string.settings)) {
							navController.goFromRoot(Route.Settings)
						}
					},
				)
			}
			currentRoute.startsWith(Route.Settings::class.qualifiedName!!) -> NavBarState(
				title = { NavTitle(stringResource(R.string.settings)) },
				show = true,
				leading = {
					NavIcon(Icons.AutoMirrored.Filled.ArrowBack, stringResource(R.string.back)) {
						navController.popBackStack()
					}
				},
			)
			currentRoute.startsWith(Route.EntryLocation::class.qualifiedName!!) -> NavBarState(
				show = true,
				title = { NavTitle(stringResource(R.string.entry)) },
				leading = { NavIcon(Icons.AutoMirrored.Filled.ArrowBack, stringResource(R.string.back)) { navController.popBackStack() } },
				trailing = { NavIcon(Icons.Outlined.Info, stringResource(R.string.info)) { showLocationTooltip = true } },
			)
			currentRoute.startsWith(Route.ExitLocation::class.qualifiedName!!) -> NavBarState(
				show = true,
				title = { NavTitle(stringResource(R.string.exit)) },
				leading = { NavIcon(Icons.AutoMirrored.Filled.ArrowBack, stringResource(R.string.back)) { navController.popBackStack() } },
				trailing = { NavIcon(Icons.Outlined.Info, stringResource(R.string.info)) { showLocationTooltip = true } },
			)
			currentRoute.startsWith(Route.Logs::class.qualifiedName!!) -> NavBarState(
				title = { NavTitle(stringResource(R.string.logs)) },
				show = true,
				leading = {
					NavIcon(Icons.AutoMirrored.Filled.ArrowBack, stringResource(R.string.back)) {
						navController.popBackStack()
					}
				},
			)
			currentRoute.startsWith(Route.Support::class.qualifiedName!!) -> NavBarState(
				title = { NavTitle(stringResource(R.string.support)) },
				show = true,
				leading = {
					NavIcon(Icons.AutoMirrored.Filled.ArrowBack, stringResource(R.string.back)) {
						navController.popBackStack()
					}
				},
			)
			currentRoute.startsWith(Route.Legal::class.qualifiedName!!) -> NavBarState(
				title = { NavTitle(stringResource(R.string.legal)) },
				show = true,
				leading = {
					NavIcon(Icons.AutoMirrored.Filled.ArrowBack, stringResource(R.string.back)) {
						navController.popBackStack()
					}
				},
			)
			currentRoute.startsWith(Route.Login::class.qualifiedName!!) -> NavBarState(show = false)
			currentRoute.startsWith(Route.Licenses::class.qualifiedName!!) -> NavBarState(
				title = { NavTitle(stringResource(R.string.licenses)) },
				show = true,
				leading = {
					NavIcon(Icons.AutoMirrored.Filled.ArrowBack, stringResource(R.string.back)) {
						navController.popBackStack()
					}
				},
			)
			currentRoute.startsWith(Route.Appearance::class.qualifiedName!!) -> NavBarState(
				title = { NavTitle(stringResource(R.string.appearance)) },
				show = true,
				leading = {
					NavIcon(Icons.AutoMirrored.Filled.ArrowBack, stringResource(R.string.back)) {
						navController.popBackStack()
					}
				},
			)
			currentRoute.startsWith(Route.Display::class.qualifiedName!!) -> NavBarState(
				title = { NavTitle(stringResource(R.string.display_theme)) },
				show = true,
				leading = {
					NavIcon(Icons.AutoMirrored.Filled.ArrowBack, stringResource(R.string.back)) {
						navController.popBackStack()
					}
				},
			)
			currentRoute.startsWith(Route.Language::class.qualifiedName!!) -> NavBarState(
				title = { NavTitle(stringResource(R.string.language)) },
				show = true,
				leading = {
					NavIcon(Icons.AutoMirrored.Filled.ArrowBack, stringResource(R.string.back)) {
						navController.popBackStack()
					}
				},
			)
			currentRoute.startsWith(Route.Developer::class.qualifiedName!!) -> NavBarState(
				title = { NavTitle(stringResource(R.string.developer)) },
				show = true,
				leading = {
					NavIcon(Icons.AutoMirrored.Filled.ArrowBack, stringResource(R.string.back)) {
						navController.popBackStack()
					}
				},
			)
			currentRoute.startsWith(Route.LoginScanner::class.qualifiedName!!) -> NavBarState(show = false)
			currentRoute.startsWith(Route.Permission::class.qualifiedName!!) -> NavBarState(
				title = { NavTitle(stringResource(R.string.permission_required)) },
				show = true,
				leading = {
					NavIcon(Icons.AutoMirrored.Filled.ArrowBack, stringResource(R.string.back)) {
						navController.popBackStack()
					}
				},
			)
			else -> NavBarState(show = false)
		}
	}

	Modal(show = showLocationTooltip, onDismiss = { showLocationTooltip = false }, title = {
		Text(
			stringResource(R.string.gateway_locations_title).uppercase(),
			style = CustomTypography.labelHuge,
			fontFamily = FontFamily(Font(R.font.lab_grotesque_mono)),
		)
	}, text = {
		ServerDetailsModalBody(onClick = { context.openWebUrl(context.getString(R.string.location_support_link)) })
	})

	AnimatedVisibility(
		visible = navBarState.show,
		enter = slideInVertically() + fadeIn(),
		exit = slideOutVertically() + fadeOut(),
	) {
		CenterAlignedTopAppBar(
			modifier = modifier,
			title = {
				navBarState.title()
			},
			actions = {
				navBarState.trailing()
			},
			navigationIcon = {
				navBarState.leading()
			},
			colors = TopAppBarDefaults.topAppBarColors(containerColor = MaterialTheme.colorScheme.background),
		)
	}
}
