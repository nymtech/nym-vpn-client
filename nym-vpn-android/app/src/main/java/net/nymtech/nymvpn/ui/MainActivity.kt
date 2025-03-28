package net.nymtech.nymvpn.ui

import android.content.Intent
import android.graphics.Color.TRANSPARENT
import android.os.Build
import android.os.Bundle
import androidx.activity.SystemBarStyle
import androidx.activity.compose.setContent
import androidx.activity.enableEdgeToEdge
import androidx.activity.viewModels
import androidx.appcompat.app.AppCompatActivity
import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.outlined.Settings
import androidx.compose.material3.Scaffold
import androidx.compose.material3.SnackbarData
import androidx.compose.material3.SnackbarDuration
import androidx.compose.material3.SnackbarHost
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.ExperimentalComposeUiApi
import androidx.compose.ui.Modifier
import androidx.compose.ui.layout.onGloballyPositioned
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.semantics.testTagsAsResourceId
import androidx.compose.ui.unit.dp
import androidx.core.splashscreen.SplashScreen.Companion.installSplashScreen
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.currentBackStackEntryAsState
import androidx.navigation.compose.rememberNavController
import androidx.navigation.toRoute
import dagger.hilt.android.AndroidEntryPoint
import kotlinx.coroutines.delay
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.nymvpn.manager.shortcut.ShortcutManager
import net.nymtech.nymvpn.ui.common.labels.CustomSnackBar
import net.nymtech.nymvpn.ui.common.navigation.LocalNavController
import net.nymtech.nymvpn.ui.common.navigation.NavBar
import net.nymtech.nymvpn.ui.common.snackbar.SnackbarController
import net.nymtech.nymvpn.ui.common.snackbar.SnackbarControllerProvider
import net.nymtech.nymvpn.ui.screens.hop.GatewayLocation
import net.nymtech.nymvpn.ui.screens.hop.HopScreen
import net.nymtech.nymvpn.ui.screens.main.MainScreen
import net.nymtech.nymvpn.ui.screens.permission.PermissionScreen
import net.nymtech.nymvpn.ui.screens.scanner.ScannerScreen
import net.nymtech.nymvpn.ui.screens.settings.SettingsScreen
import net.nymtech.nymvpn.ui.screens.settings.appearance.AppearanceScreen
import net.nymtech.nymvpn.ui.screens.settings.appearance.display.DisplayScreen
import net.nymtech.nymvpn.ui.screens.settings.appearance.language.LanguageScreen
import net.nymtech.nymvpn.ui.screens.settings.developer.DeveloperScreen
import net.nymtech.nymvpn.ui.screens.settings.legal.LegalScreen
import net.nymtech.nymvpn.ui.screens.settings.legal.licenses.LicensesScreen
import net.nymtech.nymvpn.ui.screens.settings.login.LoginScreen
import net.nymtech.nymvpn.ui.screens.settings.logs.LogsScreen
import net.nymtech.nymvpn.ui.screens.settings.support.SupportScreen
import net.nymtech.nymvpn.ui.screens.splash.SplashScreen
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.util.StringValue
import net.nymtech.nymvpn.util.extensions.isCurrentRoute
import net.nymtech.nymvpn.util.extensions.requestTileServiceStateUpdate
import net.nymtech.nymvpn.util.extensions.resetTile
import javax.inject.Inject
import kotlin.system.exitProcess

@AndroidEntryPoint
class MainActivity : AppCompatActivity() {

	@Inject
	lateinit var shortcutManager: ShortcutManager

	@Inject
	lateinit var settingsRepository: SettingsRepository

	override fun onCreate(savedInstanceState: Bundle?) {
		val appViewModel by viewModels<AppViewModel>()

		installSplashScreen().setKeepOnScreenCondition { false }

		enableEdgeToEdge(
			statusBarStyle = SystemBarStyle.auto(TRANSPARENT, TRANSPARENT),
			navigationBarStyle = SystemBarStyle.auto(TRANSPARENT, TRANSPARENT),
		)

		if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
			window.isNavigationBarContrastEnforced = false
		}

		super.onCreate(savedInstanceState)

		resetTile()

		setContent {
			val appState by appViewModel.uiState.collectAsStateWithLifecycle(lifecycle)
			val systemMessage by appViewModel.systemMessage.collectAsStateWithLifecycle(lifecycle)
			val configurationChange by appViewModel.configurationChange.collectAsStateWithLifecycle(lifecycle)

			val navController = rememberNavController()
			val navBackStackEntry by navController.currentBackStackEntryAsState()
			var navHeight by remember { mutableStateOf(0.dp) }
			val density = LocalDensity.current

			LaunchedEffect(configurationChange) {
				if (configurationChange) {
					// Restart activity for built-in translation of country names
					Intent(this@MainActivity, MainActivity::class.java).also {
						startActivity(it)
						exitProcess(0)
					}
				}
			}

			// only display system message on main screen
			LaunchedEffect(systemMessage, navBackStackEntry) {
				if (navBackStackEntry.isCurrentRoute(Route.Main::class)) {
					// delay to allow other messages before we show persistent again
					delay(2000)
					systemMessage?.let {
						SnackbarController.showMessage(StringValue.DynamicString(it.message), duration = SnackbarDuration.Indefinite)
					}
				} else if (systemMessage != null) SnackbarController.dismiss()
			}

			with(appState.settings) {
				LaunchedEffect(vpnMode, entryPoint, exitPoint) {
					this@MainActivity.requestTileServiceStateUpdate()
				}
				LaunchedEffect(isShortcutsEnabled) {
					if (!isShortcutsEnabled) return@LaunchedEffect shortcutManager.removeShortcuts()
					shortcutManager.addShortcuts()
				}
			}

			CompositionLocalProvider(LocalNavController provides navController) {
				SnackbarControllerProvider { host ->
					NymVPNTheme(theme = appState.settings.theme ?: Theme.default()) {
						Scaffold(
							contentWindowInsets = WindowInsets(0.dp),
							modifier = Modifier.semantics {
								// Enables testTag -> UiAutomator resource id
								@OptIn(ExperimentalComposeUiApi::class)
								testTagsAsResourceId = true
							},
							topBar = {
								NavBar(
									navController,
									Modifier.onGloballyPositioned {
										navHeight = with(density) {
											it.size.height.toDp()
										}
									},
								)
							},
							snackbarHost = {
								SnackbarHost(host) { snackbarData: SnackbarData ->
									CustomSnackBar(message = snackbarData.visuals.message, paddingTop = navHeight)
								}
							},
						) { padding ->
							NavHost(
								navController,
								startDestination = Route.Splash,
								modifier =
								Modifier
									.fillMaxSize()
									.padding(padding),
								enterTransition = { fadeIn(tween(200)) },
								exitTransition = { fadeOut(tween(200)) },
								popEnterTransition = { fadeIn(tween(200)) },
								popExitTransition = { fadeOut(tween(200)) },
							) {
								composable<Route.Splash> {
									SplashScreen(appViewModel, appState)
								}
								composable<Route.Main>(
									enterTransition = { fadeIn() },
									exitTransition = { fadeOut() },
								) {
									val args = it.toRoute<Route.Main>()
									MainScreen(appState, args.autoStart)
								}
								composable<Route.Permission> {
									val args = it.toRoute<Route.Permission>()
									runCatching {
										PermissionScreen(args.permission)
									}
								}
								composable<Route.Settings>(
									enterTransition = { fadeIn() },
									exitTransition = { fadeOut() },
								) {
									SettingsScreen(
										appViewModel,
										appState,
									)
								}
								composable<Route.EntryLocation> {
									HopScreen(
										gatewayLocation = GatewayLocation.ENTRY,
										appViewModel,
										appState,
									)
								}
								composable<Route.ExitLocation> {
									HopScreen(
										gatewayLocation = GatewayLocation.EXIT,
										appViewModel,
										appState,
									)
								}
								composable<Route.Logs> { LogsScreen() }
								composable<Route.Support> { SupportScreen() }
								composable<Route.Legal> { LegalScreen() }
								composable<Route.Login>(
									enterTransition = { fadeIn() },
									exitTransition = { fadeOut() },
								) {
									LoginScreen(appState)
								}
								composable<Route.Licenses> {
									LicensesScreen()
								}
								composable<Route.Appearance> {
									AppearanceScreen()
								}
								composable<Route.Display> {
									DisplayScreen(appState)
								}
								composable<Route.Language> {
									LanguageScreen(appState, appViewModel)
								}
								composable<Route.Developer> {
									DeveloperScreen(appState, appViewModel)
								}
								composable<Route.LoginScanner> {
									ScannerScreen()
								}
							}
						}
					}
				}
			}
		}
	}
}
