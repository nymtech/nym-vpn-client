package net.nymtech.nymvpn.ui

import android.content.Intent
import android.graphics.Color.TRANSPARENT
import android.net.Uri
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
import androidx.compose.material3.Scaffold
import androidx.compose.material3.SnackbarDuration
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
import androidx.lifecycle.lifecycleScope
import androidx.navigation.NavHostController
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.currentBackStackEntryAsState
import androidx.navigation.compose.rememberNavController
import androidx.navigation.toRoute
import dagger.hilt.android.AndroidEntryPoint
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.nymvpn.manager.billing.BillingManager
import net.nymtech.nymvpn.manager.shortcut.ShortcutManager
import net.nymtech.nymvpn.ui.common.navigation.LocalNavController
import net.nymtech.nymvpn.ui.common.navigation.NavBar
import net.nymtech.nymvpn.ui.common.navigation.NavBarEvent
import net.nymtech.nymvpn.ui.common.snackbar.SnackbarController
import net.nymtech.nymvpn.ui.common.snackbar.SnackbarControllerProvider
import net.nymtech.nymvpn.ui.screens.account.generating.GeneratingScreen
import net.nymtech.nymvpn.ui.screens.account.info.AccountInfoScreen
import net.nymtech.nymvpn.ui.screens.account.passphrase.PassphraseScreen
import net.nymtech.nymvpn.ui.screens.account.payment.PaymentScreen
import net.nymtech.nymvpn.ui.screens.account.plan.SelectPlanScreen
import net.nymtech.nymvpn.ui.screens.details.DetailsScreen
import net.nymtech.nymvpn.ui.screens.onboarding.OnboardingScreen
import net.nymtech.nymvpn.ui.screens.server.GatewayLocation
import net.nymtech.nymvpn.ui.screens.server.ServerScreen
import net.nymtech.nymvpn.ui.screens.main.MainScreen
import net.nymtech.nymvpn.ui.screens.permission.PermissionScreen
import net.nymtech.nymvpn.ui.screens.settings.SettingsScreen
import net.nymtech.nymvpn.ui.screens.settings.appearance.AppearanceScreen
import net.nymtech.nymvpn.ui.screens.settings.appearance.appicon.AppIconScreen
import net.nymtech.nymvpn.ui.screens.settings.appearance.display.DisplayScreen
import net.nymtech.nymvpn.ui.screens.settings.appearance.language.LanguageScreen
import net.nymtech.nymvpn.ui.screens.settings.censorship.CensorshipScreen
import net.nymtech.nymvpn.ui.screens.settings.developer.DeveloperScreen
import net.nymtech.nymvpn.ui.screens.settings.diagnostic.DiagnosticScreen
import net.nymtech.nymvpn.ui.screens.settings.dns.DnsScreen
import net.nymtech.nymvpn.ui.screens.settings.legal.LegalScreen
import net.nymtech.nymvpn.ui.screens.settings.legal.licenses.LicensesScreen
import net.nymtech.nymvpn.ui.screens.settings.logs.LogsScreen
import net.nymtech.nymvpn.ui.screens.settings.geoexclusion.GeoExclusionScreen
import net.nymtech.nymvpn.ui.screens.settings.geoexclusion.setup.SetupScreen
import net.nymtech.nymvpn.ui.screens.settings.notifications.NotificationsScreen
import net.nymtech.nymvpn.ui.screens.settings.privacy.PrivacyScreen
import net.nymtech.nymvpn.ui.screens.settings.support.SupportScreen
import net.nymtech.nymvpn.ui.screens.settings.tuning.MixnetTuningScreen
import net.nymtech.nymvpn.ui.screens.settings.tunneling.SplitTunnelingScreen
import net.nymtech.nymvpn.ui.screens.splash.SplashScreen
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.util.StringValue
import net.nymtech.nymvpn.util.extensions.goFromRoot
import net.nymtech.nymvpn.util.extensions.isCurrentRoute
import net.nymtech.nymvpn.util.extensions.navigateAndForgetToMain
import net.nymtech.nymvpn.util.extensions.requestTileServiceStateUpdate
import net.nymtech.nymvpn.util.extensions.resetTile
import timber.log.Timber
import javax.inject.Inject

@AndroidEntryPoint
class MainActivity : AppCompatActivity() {

	@Inject lateinit var shortcutManager: ShortcutManager

	@Inject lateinit var settingsRepository: SettingsRepository

	@Inject lateinit var billingManager: BillingManager

	private var pendingDeepLink: Uri? = null
	private var navControllerRef: NavHostController? = null

	val appViewModel by viewModels<AppViewModel>()

	override fun onCreate(savedInstanceState: Bundle?) {
		installSplashScreen().setKeepOnScreenCondition { false }
		enableEdgeToEdge(
			statusBarStyle = SystemBarStyle.auto(TRANSPARENT, TRANSPARENT),
			navigationBarStyle = SystemBarStyle.auto(TRANSPARENT, TRANSPARENT),
		)

		if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.Q) {
			window.isNavigationBarContrastEnforced = false
		}
		super.onCreate(savedInstanceState)

		appViewModel.onAppStartup()
		captureDeepLink(intent)
		resetTile()

		setContent {
			val appState by appViewModel.uiState.collectAsStateWithLifecycle(lifecycle)
			val systemMessage by appViewModel.systemMessage.collectAsStateWithLifecycle(lifecycle)
			val configurationChange by appViewModel.configurationChange.collectAsStateWithLifecycle(lifecycle)

			val navController = rememberNavController()
			val navBackStackEntry by navController.currentBackStackEntryAsState()

			var navHeight by remember { mutableStateOf(0.dp) }
			val density = LocalDensity.current

			var hideBackButtonInNavBar by remember { mutableStateOf(false) }
			var onBackClickEventFromRoute by remember { mutableStateOf<Route?>(null) }
			var serverLocationIsExit by remember { mutableStateOf(false) }

			var navBarEvent by remember { mutableStateOf<NavBarEvent?>(null) }

			val consumeNavBarEvent = remember {
				{ navBarEvent = null }
			}

			LaunchedEffect(navController) {
				navControllerRef = navController
				consumeDeepLinkIfAny()
			}

			LaunchedEffect(configurationChange) {
				if (configurationChange) {
					val restartIntent = Intent(this@MainActivity, MainActivity::class.java).apply {
						addFlags(Intent.FLAG_ACTIVITY_CLEAR_TOP or Intent.FLAG_ACTIVITY_NEW_TASK)
					}
					startActivity(restartIntent)
					finish()
					appViewModel.onConfigurationHandled()
				}
			}

			LaunchedEffect(systemMessage, navBackStackEntry) {
				if (navBackStackEntry.isCurrentRoute(Route.Main::class)) {
					delay(2000)
					systemMessage?.let {
						SnackbarController.showMessage(
							StringValue.DynamicString(it.message),
							duration = SnackbarDuration.Indefinite,
						)
					}
				} else if (systemMessage != null) {
					SnackbarController.dismiss()
				}
			}

			with(appState.vpnConfig) {
				LaunchedEffect(mode, entryPoint, exitPoint) {
					this@MainActivity.requestTileServiceStateUpdate()
				}
			}
			with(appState.settings) {
				LaunchedEffect(isShortcutsEnabled) {
					if (!isShortcutsEnabled) return@LaunchedEffect shortcutManager.removeShortcuts()
					shortcutManager.addShortcuts()
				}
			}

			CompositionLocalProvider(LocalNavController provides navController) {
				SnackbarControllerProvider { host, content ->
					NymVPNTheme(theme = appState.settings.theme ?: Theme.default()) {
						Scaffold(
							contentWindowInsets = WindowInsets(0.dp),
							modifier = Modifier.semantics {
								@OptIn(ExperimentalComposeUiApi::class)
								testTagsAsResourceId = true
							},
							topBar = {
								NavBar(
									navController = navController,
									modifier = Modifier.onGloballyPositioned {
										navHeight = with(density) { it.size.height.toDp() }
									},
									hideBackButton = hideBackButtonInNavBar,
									onBackClick = { onBackClickEventFromRoute = it },
									onNavBarEvent = { navBarEvent = it },
									serverLocationIsExit = serverLocationIsExit,
									logsEnabled = appState.settings.logsEnabled,
									onMainThemeClick = { navController.goFromRoot(Route.Display) },
									onMainSettingsClick = { navController.goFromRoot(Route.Settings(false)) },
								)
							},
						) { padding ->
							NavHost(
								navController = navController,
								startDestination = Route.Splash,
								modifier = Modifier
									.fillMaxSize()
									.padding(padding),
								enterTransition = { fadeIn(tween(200)) },
								exitTransition = { fadeOut(tween(200)) },
								popEnterTransition = { fadeIn(tween(200)) },
								popExitTransition = { fadeOut(tween(200)) },
							) {
								composable<Route.Splash>(
									exitTransition = { fadeOut(tween(150)) },
									popEnterTransition = { fadeIn(tween(200)) },
								) { SplashScreen(appViewModel, appState, topOffset = padding.calculateTopPadding()) }

								composable<Route.Onboarding>(
									enterTransition = { fadeIn(tween(200)) },
									exitTransition = { fadeOut(tween(150)) },
								) { OnboardingScreen() }

								composable<Route.Main>(
									enterTransition = { fadeIn() },
									exitTransition = { fadeOut() },
								) {
									val args = it.toRoute<Route.Main>()
									MainScreen(
										appViewModel,
										appState,
										args.autoStart,
										authRoute = AuthRoute.fromName(args.authRoute),
										loginProcessing = args.loginProcessing,
									)
								}

								composable<Route.Permission> {
									val args = runCatching { it.toRoute<Route.Permission>() }.getOrNull()
									if (args != null) {
										PermissionScreen(args.permission)
									} else {
										LaunchedEffect(Unit) {
											Timber.e("Failed to parse Route.Permission arguments")
											navController.popBackStack()
										}
									}
								}

								composable<Route.Settings>(
									enterTransition = { fadeIn() },
									exitTransition = { fadeOut() },
								) {
									val args = it.toRoute<Route.Settings>()
									SettingsScreen(appState, appViewModel, args.showVpnSettings)
								}

								composable<Route.EntryServer> {
									ServerScreen(
										gatewayLocation = GatewayLocation.ENTRY,
										appUiState = appState,
										navBarEvent = navBarEvent,
										onNavBarEventConsume = consumeNavBarEvent,
										onLocationChange = { serverLocationIsExit = it == GatewayLocation.EXIT },
									)
								}

								composable<Route.ExitServer> {
									ServerScreen(
										gatewayLocation = GatewayLocation.EXIT,
										appUiState = appState,
										navBarEvent = navBarEvent,
										onNavBarEventConsume = consumeNavBarEvent,
										onLocationChange = { serverLocationIsExit = it == GatewayLocation.EXIT },
									)
								}

								composable<Route.Logs> {
									LogsScreen(
										appUiState = appState,
										navBarEvent = navBarEvent,
										onNavBarEventConsume = consumeNavBarEvent,
									)
								}

								composable<Route.Support> { SupportScreen() }
								composable<Route.Legal> { LegalScreen() }

								composable<Route.Licenses> { LicensesScreen() }
								composable<Route.Censorship> { CensorshipScreen(appState) }

								composable<Route.Dns> {
									DnsScreen(
										appUiState = appState,
										onBackEventConsume = { onBackClickEventFromRoute = null },
										onBackClickEventTriggered = onBackClickEventFromRoute == Route.Dns,
									)
								}

								composable<Route.Appearance> { AppearanceScreen() }
								composable<Route.Privacy> { PrivacyScreen(appState) }
								composable<Route.Display> { DisplayScreen(appState) }
								composable<Route.AppIcon> { AppIconScreen(appState) }
								composable<Route.Language> { LanguageScreen(appState, appViewModel) }
								composable<Route.Developer> { DeveloperScreen(appState, appViewModel) }
								composable<Route.SelectPlan> { SelectPlanScreen(appViewModel) }

								composable<Route.Generating> {
									GeneratingScreen()
								}

								composable<Route.ServerDetails> {
									val args = runCatching { it.toRoute<Route.ServerDetails>() }.getOrNull()
									if (args != null) {
										DetailsScreen(appState, args.id, args.location)
									} else {
										LaunchedEffect(Unit) {
											Timber.e("Failed to parse Route.ServerDetails arguments")
											navController.popBackStack()
										}
									}
								}

								composable<Route.Payment> {
									val args = runCatching { it.toRoute<Route.Payment>() }.getOrNull()
									if (args != null) {
										PaymentScreen(appState, args.productId)
									} else {
										LaunchedEffect(Unit) {
											Timber.e("Failed to parse Route.Payment arguments")
											navController.popBackStack()
										}
									}
								}

								composable<Route.Passphrase> {
									PassphraseScreen(
										onBackButtonVisibilityChange = { hideBackButtonInNavBar = it },
										navBarEvent = navBarEvent,
										onNavBarEventConsume = consumeNavBarEvent,
									)
								}

								composable<Route.Account> { AccountInfoScreen(appViewModel, appState) }

								composable<Route.SplitTunneling> {
									SplitTunnelingScreen(
										onBackEventConsume = { onBackClickEventFromRoute = null },
										onBackClickEventTriggered = onBackClickEventFromRoute == Route.SplitTunneling,
										navBarEvent = navBarEvent,
										onNavBarEventConsume = consumeNavBarEvent,
									)
								}

								composable<Route.MixnetTuning> { MixnetTuningScreen(appState) }

								composable<Route.Diagnostic> { DiagnosticScreen() }
								composable<Route.Notifications> { NotificationsScreen(appState) }
								composable<Route.GeoExclusion> { GeoExclusionScreen(appState) }
								composable<Route.Setup> { SetupScreen(appState) }
							}
						}
					}
				}
			}
		}
	}

	override fun onNewIntent(intent: Intent) {
		super.onNewIntent(intent)
		setIntent(intent)
		captureDeepLink(intent)
		consumeDeepLinkIfAny()
	}

	override fun onDestroy() {
		super.onDestroy()
		if (isFinishing) billingManager.endConnection()
	}

	private fun captureDeepLink(intent: Intent?) {
		val uri = intent?.data ?: return
		if (uri.scheme != "nymvpn") return
		pendingDeepLink = uri
	}

	private fun consumeDeepLinkIfAny() {
		val uri = pendingDeepLink ?: return
		pendingDeepLink = null

		val navController = navControllerRef
		if (navController == null) {
			pendingDeepLink = uri
			return
		}
		handleDeepLink(uri)
	}

	private fun handleDeepLink(uri: Uri) {
		val host = uri.host
		val path = uri.path
		if (host == "auth" && path?.startsWith("/privy/privateKey") == true) {
			lifecycleScope.launch {
				val storeSucceeded = appViewModel.handleDeepLinkAuth(uri.toString())
				navControllerRef?.navigateAndForgetToMain(routeAfterDeepLinkAuth(storeSucceeded))
			}
		} else if (host == "account" && path?.startsWith("/response") == true) {
			lifecycleScope.launch {
				appViewModel.handleDeepLinkAuth(uri.toString())
				appViewModel.dismissAutologin()
				navControllerRef?.navigateAndForgetToMain(Route.Main(autoStart = false))
			}
		}
	}

	private fun routeAfterDeepLinkAuth(storeSucceeded: Boolean): Route = if (storeSucceeded) {
		Route.Main(autoStart = false, loginProcessing = true)
	} else {
		Route.Main(autoStart = false)
	}
}
