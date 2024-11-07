package net.nymtech.nymvpn.ui

import android.content.Context
import android.content.ContextWrapper
import android.content.Intent
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Scaffold
import androidx.compose.material3.SnackbarData
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
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.ViewModelProvider
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.lifecycleScope
import androidx.lifecycle.repeatOnLifecycle
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.currentBackStackEntryAsState
import androidx.navigation.compose.rememberNavController
import androidx.navigation.toRoute
import dagger.hilt.android.AndroidEntryPoint
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import net.nymtech.localizationutil.LocaleStorage
import net.nymtech.localizationutil.LocaleUtil
import net.nymtech.nymvpn.NymVpn
import net.nymtech.nymvpn.manager.shortcut.ShortcutManager
import net.nymtech.nymvpn.service.notification.NotificationService
import net.nymtech.nymvpn.ui.common.labels.CustomSnackBar
import net.nymtech.nymvpn.ui.common.navigation.LocalNavController
import net.nymtech.nymvpn.ui.common.navigation.NavBar
import net.nymtech.nymvpn.ui.common.snackbar.SnackbarControllerProvider
import net.nymtech.nymvpn.ui.screens.analytics.AnalyticsScreen
import net.nymtech.nymvpn.ui.screens.hop.GatewayLocation
import net.nymtech.nymvpn.ui.screens.hop.HopScreen
import net.nymtech.nymvpn.ui.screens.main.MainScreen
import net.nymtech.nymvpn.ui.screens.permission.PermissionScreen
import net.nymtech.nymvpn.ui.screens.scanner.ScannerScreen
import net.nymtech.nymvpn.ui.screens.settings.SettingsScreen
import net.nymtech.nymvpn.ui.screens.settings.account.AccountScreen
import net.nymtech.nymvpn.ui.screens.settings.appearance.AppearanceScreen
import net.nymtech.nymvpn.ui.screens.settings.appearance.display.DisplayScreen
import net.nymtech.nymvpn.ui.screens.settings.appearance.language.LanguageScreen
import net.nymtech.nymvpn.ui.screens.settings.credential.CredentialScreen
import net.nymtech.nymvpn.ui.screens.settings.environment.EnvironmentScreen
import net.nymtech.nymvpn.ui.screens.settings.feedback.FeedbackScreen
import net.nymtech.nymvpn.ui.screens.settings.legal.LegalScreen
import net.nymtech.nymvpn.ui.screens.settings.legal.licenses.LicensesScreen
import net.nymtech.nymvpn.ui.screens.settings.logs.LogsScreen
import net.nymtech.nymvpn.ui.screens.settings.support.SupportScreen
import net.nymtech.nymvpn.ui.screens.splash.SplashScreen
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.util.Constants
import net.nymtech.nymvpn.util.extensions.isCurrentRoute
import net.nymtech.nymvpn.util.extensions.navigateAndForget
import net.nymtech.nymvpn.util.extensions.requestTileServiceStateUpdate
import net.nymtech.nymvpn.util.extensions.resetTile
import net.nymtech.vpn.model.BackendMessage
import timber.log.Timber
import java.util.Locale
import javax.inject.Inject

@AndroidEntryPoint
class MainActivity : ComponentActivity() {

	val localeStorage: LocaleStorage by lazy {
		LocaleStorage(this)
	}

	@Inject
	lateinit var notificationService: NotificationService

	@Inject
	lateinit var shortcutManager: ShortcutManager

	private lateinit var appViewModel: AppViewModel

	override fun onCreate(savedInstanceState: Bundle?) {
		super.onCreate(savedInstanceState)

		appViewModel = ViewModelProvider(this)[AppViewModel::class.java]

		this.resetTile()

		lifecycleScope.launch {
			repeatOnLifecycle(Lifecycle.State.CREATED) {
				appViewModel.onAppStartup()
			}
		}

		setContent {
			val appState by appViewModel.uiState.collectAsStateWithLifecycle(lifecycle = this.lifecycle)
			val navBarState by appViewModel.navBarState.collectAsStateWithLifecycle(lifecycle = this.lifecycle)

			val navController = rememberNavController()
			val navBackStackEntry by navController.currentBackStackEntryAsState()
			var navHeight by remember { mutableStateOf(0.dp) }
			val density = LocalDensity.current

			LaunchedEffect(navBackStackEntry) {
				if (navBackStackEntry.isCurrentRoute(Route.Main::class)) {
					val args = navBackStackEntry?.toRoute<Route.Main>()
					if(args?.changeLanguage == true) {
						val lang = localeStorage.getPreferredLocale()
						setLocale(lang)
					}
				}
			}

			with(appState.settings) {
				LaunchedEffect(vpnMode, lastHopCountry, firstHopCountry) {
					this@MainActivity.requestTileServiceStateUpdate()
				}
				LaunchedEffect(isShortcutsEnabled) {
					if (!isShortcutsEnabled) return@LaunchedEffect shortcutManager.removeShortcuts()
					shortcutManager.addShortcuts()
				}
			}

			LaunchedEffect(appState.backendMessage) {
				when (val message = appState.backendMessage) {
					is BackendMessage.Failure -> {
						// TODO invalid credential errors?
// 						when (message.exception) {
// 							is VpnException.InvalidCredential -> {
// 								if (NymVpn.isForeground()) {
// 									SnackbarController.showMessage(StringValue.StringResource(R.string.exception_cred_invalid))
// 									navController.goFromRoot(Route.Credential)
// 								}
// 							}
//
// 							else -> Unit
// 						}
					}

					else -> Unit
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
									navBarState,
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
								composable<Route.Main> {
									val args = it.toRoute<Route.Main>()
									MainScreen(appViewModel, appState, args.autoStart)
								}
								composable<Route.Analytics> { AnalyticsScreen(appViewModel, navController, appState) }
								composable<Route.Permission> {
									val args = it.toRoute<Route.Permission>()
									runCatching {
										PermissionScreen(appViewModel, args.permission)
									}
								}
								composable<Route.Settings> {
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
								composable<Route.Logs> { LogsScreen(appViewModel) }
								composable<Route.Support> { SupportScreen(appViewModel) }
								composable<Route.Feedback> { FeedbackScreen(appViewModel) }
								composable<Route.Legal> { LegalScreen(appViewModel) }
								composable<Route.Credential> {
									CredentialScreen(appState, appViewModel)
								}
								composable<Route.Account> { AccountScreen(appViewModel, appState) }
								composable<Route.Licenses> {
									LicensesScreen(appViewModel)
								}
								composable<Route.Appearance> {
									AppearanceScreen(appViewModel)
								}
								composable<Route.Display> {
									DisplayScreen(appState, appViewModel)
								}
								composable<Route.Language> {
									LanguageScreen(appViewModel, localeStorage)
								}
								composable<Route.Environment> {
									EnvironmentScreen(appState, appViewModel)
								}
								composable<Route.CredentialScanner> {
									ScannerScreen()
								}
							}
						}
					}
				}
			}
		}
	}

	override fun attachBaseContext(newBase: Context) {
		val lang = LocaleStorage(newBase).getPreferredLocale()
		val context = updateLocale(newBase, lang)
		super.attachBaseContext(context)
	}

	fun updateLocale(context: Context, lang: String): ContextWrapper {
		val locale = LocaleUtil.getLocaleFromPrefCode(lang)
		Locale.setDefault(locale)
		val resources = context.resources
		val configuration = resources.configuration
		configuration.setLocale(locale)
		configuration.setLayoutDirection(locale)
		return ContextWrapper(context.createConfigurationContext(configuration))
	}

	private fun setLocale(localeName: String) {
		val locale = Locale(localeName)
		Locale.setDefault(locale)
		resources.updateConfiguration(resources.configuration, resources.displayMetrics)

		// Restart activity to apply the new locale
		val refresh = Intent(this, MainActivity::class.java)
		finish()
		startActivity(refresh)
	}
}
