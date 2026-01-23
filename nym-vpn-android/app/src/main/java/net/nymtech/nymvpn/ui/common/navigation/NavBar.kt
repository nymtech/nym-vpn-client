package net.nymtech.nymvpn.ui.common.navigation

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.slideInVertically
import androidx.compose.animation.slideOutVertically
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.filled.Close
import androidx.compose.material.icons.outlined.Info
import androidx.compose.material.icons.outlined.Settings
import androidx.compose.material3.CenterAlignedTopAppBar
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.TopAppBarDefaults
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalSoftwareKeyboardController
import androidx.compose.ui.res.stringResource
import androidx.navigation.NavController
import androidx.navigation.compose.currentBackStackEntryAsState
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.Route
import net.nymtech.nymvpn.ui.screens.settings.logs.modal.LogsActionsMenu
import net.nymtech.nymvpn.util.extensions.goFromRoot
import net.nymtech.nymvpn.util.extensions.replaceCurrentWith
import net.nymtech.nymvpn.util.extensions.safePopBackStack

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun NavBar(
	navController: NavController,
	modifier: Modifier = Modifier,
	hideBackButton: Boolean = false,
	onBackClick: (Route) -> Unit = {},
	onNavBarEvent: (NavBarEvent) -> Unit = {},
	logsEnabled: Boolean = false,
) {
	val keyboardController = LocalSoftwareKeyboardController.current

	val navBackStackEntry by navController.currentBackStackEntryAsState()
	var navBarState by remember { mutableStateOf(NavBarState()) }

	LaunchedEffect(navBackStackEntry, hideBackButton, logsEnabled) {
		keyboardController?.hide()
		val currentRoute = navBackStackEntry?.destination?.route ?: return@LaunchedEffect

		navBarState = when {
			currentRoute.startsWith(Route.Splash::class.qualifiedName!!) -> NavBarState(show = false)

			currentRoute.startsWith(Route.Main::class.qualifiedName!!) -> NavBarState(
				title = { MainTitle() },
				show = true,
				trailing = {
					NavIcon(Icons.Outlined.Settings, stringResource(R.string.settings)) {
						navController.goFromRoot(Route.Settings(false))
					}
				},
			)

			currentRoute.startsWith(Route.Settings::class.qualifiedName!!) -> NavBarState(
				title = { NavTitle(stringResource(R.string.settings)) },
				show = true,
				leading = {
					NavIcon(Icons.AutoMirrored.Filled.ArrowBack, stringResource(R.string.back)) {
						navController.safePopBackStack()
					}
				},
			)

			currentRoute.startsWith(Route.EntryLocation::class.qualifiedName!!) -> NavBarState(
				show = true,
				title = { NavTitle(stringResource(R.string.entry_location)) },
				leading = {
					NavIcon(Icons.AutoMirrored.Filled.ArrowBack, stringResource(R.string.back)) {
						navController.safePopBackStack()
					}
				},
				trailing = {
					NavIcon(Icons.Outlined.Info, stringResource(R.string.info)) {
						onNavBarEvent(NavBarEvent.EntryLocationInfoClicked)
					}
				},
			)

			currentRoute.startsWith(Route.ExitLocation::class.qualifiedName!!) -> NavBarState(
				show = true,
				title = { NavTitle(stringResource(R.string.exit_location)) },
				leading = {
					NavIcon(Icons.AutoMirrored.Filled.ArrowBack, stringResource(R.string.back)) {
						navController.safePopBackStack()
					}
				},
				trailing = {
					NavIcon(Icons.Outlined.Info, stringResource(R.string.info)) {
						onNavBarEvent(NavBarEvent.ExitLocationInfoClicked)
					}
				},
			)

			currentRoute.startsWith(Route.Logs::class.qualifiedName!!) -> NavBarState(
				title = { NavTitle(stringResource(R.string.logs)) },
				show = true,
				leading = {
					NavIcon(Icons.AutoMirrored.Filled.ArrowBack, stringResource(R.string.back)) {
						navController.safePopBackStack()
					}
				},
				trailing = {
					if (logsEnabled) {
						LogsActionsMenu(
							onDownload = { onNavBarEvent(NavBarEvent.LogsDownloadClicked) },
							onShare = { onNavBarEvent(NavBarEvent.LogsShareClicked) },
							onDelete = { onNavBarEvent(NavBarEvent.LogsDeleteClicked) },
						)
					}
				},
			)

			currentRoute.startsWith(Route.Support::class.qualifiedName!!) -> NavBarState(
				title = { NavTitle(stringResource(R.string.support)) },
				show = true,
				leading = {
					NavIcon(Icons.AutoMirrored.Filled.ArrowBack, stringResource(R.string.back)) {
						navController.safePopBackStack()
					}
				},
			)

			currentRoute.startsWith(Route.Legal::class.qualifiedName!!) -> NavBarState(
				title = { NavTitle(stringResource(R.string.legal)) },
				show = true,
				leading = {
					NavIcon(Icons.AutoMirrored.Filled.ArrowBack, stringResource(R.string.back)) {
						navController.safePopBackStack()
					}
				},
			)

			currentRoute.startsWith(Route.LoginScanner::class.qualifiedName!!) -> NavBarState(show = false)

			currentRoute.startsWith(Route.Login::class.qualifiedName!!) -> NavBarState(
				title = { MainTitle() },
				show = true,
				leading = {
					NavIcon(Icons.AutoMirrored.Filled.ArrowBack, stringResource(R.string.back)) {
						navController.safePopBackStack()
					}
				},
			)

			currentRoute.startsWith(Route.Licenses::class.qualifiedName!!) -> NavBarState(
				title = { NavTitle(stringResource(R.string.licenses)) },
				show = true,
				leading = {
					NavIcon(Icons.AutoMirrored.Filled.ArrowBack, stringResource(R.string.back)) {
						navController.safePopBackStack()
					}
				},
			)

			currentRoute.startsWith(Route.Censorship::class.qualifiedName!!) -> NavBarState(
				title = { NavTitle(stringResource(R.string.settings_censorship_title)) },
				show = true,
				leading = {
					NavIcon(Icons.AutoMirrored.Filled.ArrowBack, stringResource(R.string.back)) {
						navController.safePopBackStack()
					}
				},
			)

			currentRoute.startsWith(Route.Dns::class.qualifiedName!!) -> NavBarState(
				title = { NavTitle(stringResource(R.string.dns_title)) },
				show = true,
				leading = {
					NavIcon(Icons.AutoMirrored.Filled.ArrowBack, stringResource(R.string.back)) {
						onBackClick(Route.Dns)
					}
				},
			)

			currentRoute.startsWith(Route.Appearance::class.qualifiedName!!) -> NavBarState(
				title = { NavTitle(stringResource(R.string.appearance)) },
				show = true,
				leading = {
					NavIcon(Icons.AutoMirrored.Filled.ArrowBack, stringResource(R.string.back)) {
						navController.safePopBackStack()
					}
				},
			)

			currentRoute.startsWith(Route.Privacy::class.qualifiedName!!) -> NavBarState(
				title = { NavTitle(stringResource(R.string.privacy_title)) },
				show = true,
				leading = {
					NavIcon(Icons.AutoMirrored.Filled.ArrowBack, stringResource(R.string.back)) {
						navController.safePopBackStack()
					}
				},
			)

			currentRoute.startsWith(Route.Display::class.qualifiedName!!) -> NavBarState(
				title = { NavTitle(stringResource(R.string.display_theme)) },
				show = true,
				leading = {
					NavIcon(Icons.AutoMirrored.Filled.ArrowBack, stringResource(R.string.back)) {
						navController.safePopBackStack()
					}
				},
			)

			currentRoute.startsWith(Route.SelectPlan::class.qualifiedName!!) -> NavBarState(
				title = { MainTitle() },
				show = true,
				leading = {
					NavIcon(Icons.AutoMirrored.Filled.ArrowBack, stringResource(R.string.back)) {
						navController.replaceCurrentWith(Route.Main())
					}
				},
			)

			currentRoute.startsWith(Route.CreateAccount::class.qualifiedName!!) -> NavBarState(
				title = { MainTitle() },
				show = true,
				leading = {
					NavIcon(Icons.AutoMirrored.Filled.ArrowBack, stringResource(R.string.back)) {
						navController.safePopBackStack()
					}
				},
			)

			currentRoute.startsWith(Route.Generating::class.qualifiedName!!) -> NavBarState(
				title = { NavTitle("") },
				show = true,
				leading = {},
			)

			currentRoute.startsWith(Route.Payment::class.qualifiedName!!) -> NavBarState(
				title = { NavTitle("") },
				show = true,
				leading = {},
			)

			currentRoute.startsWith(Route.Language::class.qualifiedName!!) -> NavBarState(
				title = { NavTitle(stringResource(R.string.language)) },
				show = true,
				leading = {
					NavIcon(Icons.AutoMirrored.Filled.ArrowBack, stringResource(R.string.back)) {
						navController.safePopBackStack()
					}
				},
			)

			currentRoute.startsWith(Route.Developer::class.qualifiedName!!) -> NavBarState(
				title = { NavTitle(stringResource(R.string.developer)) },
				show = true,
				leading = {
					NavIcon(Icons.AutoMirrored.Filled.ArrowBack, stringResource(R.string.back)) {
						navController.safePopBackStack()
					}
				},
			)

			currentRoute.startsWith(Route.Permission::class.qualifiedName!!) -> NavBarState(
				title = { NavTitle(stringResource(R.string.permission_required)) },
				show = true,
				leading = {
					NavIcon(Icons.AutoMirrored.Filled.ArrowBack, stringResource(R.string.back)) {
						navController.safePopBackStack()
					}
				},
			)

			currentRoute.startsWith(Route.ServerDetails::class.qualifiedName!!) -> NavBarState(
				title = { NavTitle(stringResource(R.string.details_title)) },
				show = true,
				leading = {
					NavIcon(Icons.AutoMirrored.Filled.ArrowBack, stringResource(R.string.back)) {
						navController.safePopBackStack()
					}
				},
			)

			currentRoute.startsWith(Route.Technical::class.qualifiedName!!) -> NavBarState(show = false)

			currentRoute.startsWith(Route.Passphrase::class.qualifiedName!!) -> NavBarState(
				title = { NavTitle(stringResource(R.string.settings_passphrase_title)) },
				show = true,
				leading = {
					if (!hideBackButton) {
						NavIcon(Icons.AutoMirrored.Filled.ArrowBack, stringResource(R.string.back)) {
							navController.safePopBackStack()
						}
					}
				},
				trailing = {
					NavIcon(Icons.Outlined.Info, stringResource(R.string.info)) {
						onNavBarEvent(NavBarEvent.PassphraseInfoClicked)
					}
				},
			)

			currentRoute.startsWith(Route.SplitTunneling::class.qualifiedName!!) -> NavBarState(
				title = { NavTitle(stringResource(R.string.split_tunneling)) },
				show = true,
				leading = {
					NavIcon(Icons.AutoMirrored.Filled.ArrowBack, stringResource(R.string.back)) {
						onBackClick(Route.SplitTunneling)
					}
				},
				trailing = {
					NavIcon(Icons.Outlined.Info, stringResource(R.string.info)) {
						onNavBarEvent(NavBarEvent.SplitTunnelingInfoClicked)
					}
				},
			)

			currentRoute.startsWith(Route.Account::class.qualifiedName!!) -> NavBarState(
				title = { NavTitle(stringResource(R.string.account_title)) },
				show = true,
				leading = {
					NavIcon(Icons.AutoMirrored.Filled.ArrowBack, stringResource(R.string.back)) {
						navController.safePopBackStack()
					}
				},
			)

			currentRoute.startsWith(Route.Welcome::class.qualifiedName!!) -> NavBarState(
				title = { MainTitle() },
				show = true,
				trailing = {
					NavIcon(Icons.Filled.Close, stringResource(R.string.close)) {
						navController.navigate(Route.Main())
					}
				},
			)

			else -> NavBarState(show = false)
		}
	}

	AnimatedVisibility(
		visible = navBarState.show,
		enter = slideInVertically() + fadeIn(),
		exit = slideOutVertically() + fadeOut(),
	) {
		CenterAlignedTopAppBar(
			modifier = modifier,
			title = { navBarState.title() },
			actions = { navBarState.trailing() },
			navigationIcon = { navBarState.leading() },
			colors = TopAppBarDefaults.topAppBarColors(containerColor = MaterialTheme.colorScheme.background),
		)
	}
}
