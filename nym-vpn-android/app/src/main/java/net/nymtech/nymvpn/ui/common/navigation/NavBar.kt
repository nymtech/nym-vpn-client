package net.nymtech.nymvpn.ui.common.navigation

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.slideInVertically
import androidx.compose.animation.slideOutVertically
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.filled.Close
import androidx.compose.material.icons.outlined.Contrast
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
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalSoftwareKeyboardController
import androidx.compose.ui.res.stringResource
import androidx.navigation.NavController
import androidx.navigation.compose.currentBackStackEntryAsState
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.Route
import net.nymtech.nymvpn.ui.screens.settings.logs.modal.LogsActionsMenu
import net.nymtech.nymvpn.ui.theme.LocalNymColors
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
	serverLocationIsExit: Boolean = false,
	logsEnabled: Boolean = false,
	onMainThemeClick: () -> Unit = {},
	onMainSettingsClick: () -> Unit = {},
) {
	val keyboardController = LocalSoftwareKeyboardController.current
	val navBackStackEntry by navController.currentBackStackEntryAsState()
	var navBarState: NavBarState by remember { mutableStateOf(NavBarState.Empty) }

	val currentMainThemeClick by rememberUpdatedState(onMainThemeClick)
	val currentMainSettingsClick by rememberUpdatedState(onMainSettingsClick)

	val currentRoute = navBackStackEntry?.destination?.route
	val backgroundColor = when (navBarState) {
		is NavBarState.Main -> LocalNymColors.current.navBarTitleBackground
		is NavBarState.Empty -> MaterialTheme.colorScheme.background
		else -> MaterialTheme.colorScheme.surface
	}
	LaunchedEffect(currentRoute, hideBackButton, logsEnabled, serverLocationIsExit) {
		keyboardController?.hide()
		val route = currentRoute ?: return@LaunchedEffect

		navBarState = when {
			route.startsWith(Route.Splash::class.qualifiedName!!) ||
				route.startsWith(Route.Onboarding::class.qualifiedName!!) -> NavBarState.Empty

			route.startsWith(Route.Generating::class.qualifiedName!!) ||
				route.startsWith(Route.Payment::class.qualifiedName!!) -> NavBarState.Empty

			route.startsWith(Route.Main::class.qualifiedName!!) -> NavBarState.Main(
				onThemeClick = currentMainThemeClick,
				onSettingsClick = currentMainSettingsClick,
			)

			route.startsWith(Route.SelectPlan::class.qualifiedName!!) -> NavBarState.WithBack(
				titleRes = null,
				onBack = { navController.replaceCurrentWith(Route.Main()) },
			)

			route.startsWith(Route.Settings::class.qualifiedName!!) -> NavBarState.WithClose(
				titleRes = R.string.settings,
				onClose = { navController.safePopBackStack() },
			)

			route.startsWith(Route.EntryServer::class.qualifiedName!!) ||
				route.startsWith(Route.ExitServer::class.qualifiedName!!) -> NavBarState.WithBack(
				titleRes = if (serverLocationIsExit) R.string.exit_location else R.string.entry_location,
				onBack = { navController.safePopBackStack() },
				trailing = NavBarState.Trailing.Info {
					onNavBarEvent(if (serverLocationIsExit) NavBarEvent.ExitLocationInfoClicked else NavBarEvent.EntryLocationInfoClicked)
				},
			)

			route.startsWith(Route.Logs::class.qualifiedName!!) -> NavBarState.WithBack(
				titleRes = R.string.logs,
				onBack = { navController.safePopBackStack() },
				trailing = if (logsEnabled) {
					NavBarState.Trailing.LogsMenu(
						onDownload = { onNavBarEvent(NavBarEvent.LogsDownloadClicked) },
						onShare = { onNavBarEvent(NavBarEvent.LogsShareClicked) },
						onDelete = { onNavBarEvent(NavBarEvent.LogsDeleteClicked) },
					)
				} else {
					NavBarState.Trailing.None
				},
			)

			route.startsWith(Route.Support::class.qualifiedName!!) -> NavBarState.WithBack(
				titleRes = R.string.support,
				onBack = { navController.safePopBackStack() },
			)

			route.startsWith(Route.Legal::class.qualifiedName!!) -> NavBarState.WithBack(
				titleRes = R.string.settings_legal_title,
				onBack = { navController.safePopBackStack() },
			)

			route.startsWith(Route.Licenses::class.qualifiedName!!) -> NavBarState.WithBack(
				titleRes = R.string.licenses,
				onBack = { navController.safePopBackStack() },
			)

			route.startsWith(Route.Censorship::class.qualifiedName!!) -> NavBarState.WithBack(
				titleRes = R.string.settings_censorship_title,
				onBack = { navController.safePopBackStack() },
			)

			route.startsWith(Route.Dns::class.qualifiedName!!) -> NavBarState.WithBack(
				titleRes = R.string.dns_title,
				onBack = { onBackClick(Route.Dns) },
			)

			route.startsWith(Route.Appearance::class.qualifiedName!!) -> NavBarState.WithBack(
				titleRes = R.string.appearance,
				onBack = { navController.safePopBackStack() },
			)

			route.startsWith(Route.Privacy::class.qualifiedName!!) -> NavBarState.WithBack(
				titleRes = R.string.privacy_title,
				onBack = { navController.safePopBackStack() },
			)

			route.startsWith(Route.Display::class.qualifiedName!!) -> NavBarState.WithBack(
				titleRes = R.string.display_theme,
				onBack = { navController.safePopBackStack() },
			)

			route.startsWith(Route.AppIcon::class.qualifiedName!!) -> NavBarState.WithBack(
				titleRes = R.string.app_icon_title,
				onBack = { navController.safePopBackStack() },
			)

			route.startsWith(Route.Language::class.qualifiedName!!) -> NavBarState.WithBack(
				titleRes = R.string.language,
				onBack = { navController.safePopBackStack() },
			)

			route.startsWith(Route.Developer::class.qualifiedName!!) -> NavBarState.WithBack(
				titleRes = R.string.developer,
				onBack = { navController.safePopBackStack() },
			)

			route.startsWith(Route.Permission::class.qualifiedName!!) -> NavBarState.WithBack(
				titleRes = R.string.permission_required,
				onBack = { navController.safePopBackStack() },
			)

			route.startsWith(Route.ServerDetails::class.qualifiedName!!) -> NavBarState.WithBack(
				titleRes = R.string.details_title,
				onBack = { navController.safePopBackStack() },
			)

			route.startsWith(Route.Passphrase::class.qualifiedName!!) -> NavBarState.WithBack(
				titleRes = R.string.settings_passphrase_title,
				onBack = { navController.safePopBackStack() },
				trailing = NavBarState.Trailing.Info { onNavBarEvent(NavBarEvent.PassphraseInfoClicked) },
			)

			route.startsWith(Route.SplitTunneling::class.qualifiedName!!) -> NavBarState.WithBack(
				titleRes = R.string.settings_split_tunneling_title,
				onBack = { onBackClick(Route.SplitTunneling) },
				trailing = NavBarState.Trailing.Info { onNavBarEvent(NavBarEvent.SplitTunnelingInfoClicked) },
			)

			route.startsWith(Route.Account::class.qualifiedName!!) -> NavBarState.WithBack(
				titleRes = R.string.account_title,
				onBack = { navController.safePopBackStack() },
			)

			route.startsWith(Route.MixnetTuning::class.qualifiedName!!) -> NavBarState.WithBack(
				titleRes = R.string.settings_mixnet_tuning_title,
				onBack = { navController.safePopBackStack() },
			)

			route.startsWith(Route.Diagnostic::class.qualifiedName!!) -> NavBarState.WithBack(
				titleRes = R.string.privacy_diagnostic_tool,
				onBack = { navController.safePopBackStack() },
			)
			route.startsWith(Route.Notifications::class.qualifiedName!!) -> NavBarState.WithBack(
				titleRes = R.string.settings_notifications_title,
				onBack = { navController.safePopBackStack() },
			)
			route.startsWith(Route.GeoExclusion::class.qualifiedName!!) -> NavBarState.WithBack(
				titleRes = R.string.settings_geo_exclusion_title,
				onBack = { navController.safePopBackStack() },
			)
			route.startsWith(Route.Setup::class.qualifiedName!!) -> NavBarState.WithBack(
				titleRes = R.string.setup_instructions_title,
				onBack = { navController.safePopBackStack() },
			)

			else -> NavBarState.Hidden
		}
	}

	AnimatedVisibility(
		visible = navBarState !is NavBarState.Hidden,
		enter = slideInVertically() + fadeIn(),
		exit = slideOutVertically() + fadeOut(),
	) {
		CenterAlignedTopAppBar(
			modifier = modifier,
			title = {
				when (val state = navBarState) {
					is NavBarState.Main -> MainTitle()
					is NavBarState.WithBack -> if (state.titleRes != null) NavTitle(stringResource(state.titleRes)) else MainTitle()
					is NavBarState.WithClose -> if (state.titleRes != null) NavTitle(stringResource(state.titleRes)) else MainTitle()
					else -> {}
				}
			},
			navigationIcon = {
				when (val state = navBarState) {
					is NavBarState.Main -> NavIcon(
						icon = Icons.Outlined.Contrast,
						description = stringResource(R.string.appearance),
						onClick = state.onThemeClick,
					)
					is NavBarState.WithBack -> state.onBack?.let { onBack ->
						NavIcon(
							icon = Icons.AutoMirrored.Filled.ArrowBack,
							description = stringResource(R.string.back),
							onClick = onBack,
						)
					}
					else -> {}
				}
			},
			actions = {
				when (val state = navBarState) {
					is NavBarState.Main -> NavIcon(
						icon = Icons.Outlined.Settings,
						description = stringResource(R.string.settings),
						onClick = state.onSettingsClick,
					)
					is NavBarState.WithClose -> if (state.showClose) {
						NavIcon(
							icon = Icons.Filled.Close,
							description = stringResource(R.string.close),
							onClick = state.onClose,
						)
					}
					is NavBarState.WithBack -> when (val trailing = state.trailing) {
						is NavBarState.Trailing.Info -> NavIcon(
							icon = Icons.Outlined.Info,
							description = stringResource(R.string.info),
							onClick = trailing.onClick,
						)
						is NavBarState.Trailing.LogsMenu -> LogsActionsMenu(
							onDownload = trailing.onDownload,
							onShare = trailing.onShare,
							onDelete = trailing.onDelete,
						)
						NavBarState.Trailing.None -> {}
					}
					else -> {}
				}
			},
			colors = TopAppBarDefaults.topAppBarColors(containerColor = backgroundColor),
		)
	}
}
