package net.nymtech.nymvpn.ui

import android.content.Context
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.annotation.Keep
import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Scaffold
import androidx.compose.material3.SnackbarData
import androidx.compose.material3.SnackbarHost
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.ExperimentalComposeUiApi
import androidx.compose.ui.Modifier
import androidx.compose.ui.layout.onGloballyPositioned
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.semantics.semantics
import androidx.compose.ui.semantics.testTagsAsResourceId
import androidx.compose.ui.unit.dp
import androidx.lifecycle.ViewModelProvider
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.lifecycleScope
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.currentBackStackEntryAsState
import dagger.hilt.android.AndroidEntryPoint
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import net.nymtech.localizationutil.LocaleStorage
import net.nymtech.localizationutil.LocaleUtil
import net.nymtech.nymvpn.NymVpn
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.manager.shortcut.ShortcutManager
import net.nymtech.nymvpn.service.notification.NotificationService
import net.nymtech.nymvpn.ui.common.labels.CustomSnackBar
import net.nymtech.nymvpn.ui.common.navigation.NavBar
import net.nymtech.nymvpn.ui.common.snackbar.SnackbarController
import net.nymtech.nymvpn.ui.common.snackbar.SnackbarControllerProvider
import net.nymtech.nymvpn.ui.screens.analytics.AnalyticsScreen
import net.nymtech.nymvpn.ui.screens.hop.GatewayLocation
import net.nymtech.nymvpn.ui.screens.hop.HopScreen
import net.nymtech.nymvpn.ui.screens.main.MainScreen
import net.nymtech.nymvpn.ui.screens.permission.Permission
import net.nymtech.nymvpn.ui.screens.permission.PermissionScreen
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
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.util.Constants
import net.nymtech.nymvpn.util.StringValue
import net.nymtech.nymvpn.util.extensions.go
import net.nymtech.nymvpn.util.extensions.requestTileServiceStateUpdate
import net.nymtech.nymvpn.util.extensions.resetTile
import net.nymtech.vpn.model.BackendMessage
import nym_vpn_lib.VpnException
import java.util.Locale
import javax.inject.Inject

@AndroidEntryPoint
@Keep
class MainActivity : ComponentActivity() {

	private val localeStorage: LocaleStorage by lazy {
		(application as NymVpn).localeStorage
	}

	private lateinit var oldPrefLocaleCode: String

	@Inject
	lateinit var notificationService: NotificationService

	@Inject
	lateinit var shortcutManager: ShortcutManager

	private lateinit var appViewModel: AppViewModel

	override fun onCreate(savedInstanceState: Bundle?) {
		super.onCreate(savedInstanceState)
		appViewModel = ViewModelProvider(this)[AppViewModel::class.java]
		this.resetTile()

		val isAnalyticsShown = intent.extras?.getBoolean(SplashActivity.IS_ANALYTICS_SHOWN_INTENT_KEY)
		val theme = intent.extras?.getString(SplashActivity.THEME)

		setContent {
			val appState by appViewModel.uiState.collectAsStateWithLifecycle(lifecycle = this.lifecycle)

			val navController = remember { appViewModel.navController }
			val navBackStackEntry by navController.currentBackStackEntryAsState()
			var navHeight by remember { mutableStateOf(0.dp) }
			var showNavBar by rememberSaveable { mutableStateOf(true) }
			val density = LocalDensity.current

			navController.addOnDestinationChangedListener { controller, destination, _ ->
				if (destination.route == Destination.Main.route &&
					controller.previousBackStackEntry?.destination?.route == Destination.Language.route
				) {
					val locale = LocaleUtil.getLocaleFromPrefCode(localeStorage.getPreferredLocale())
					val currentLocale = Locale.getDefault()
					if (locale != currentLocale) {
						lifecycleScope.launch {
							delay(Constants.LANGUAGE_SWITCH_DELAY)
							recreate()
						}
					}
				}
			}

			with(appState.settings) {
				LaunchedEffect(vpnMode, lastHopCountry, firstHopCountry) {
					this@MainActivity.requestTileServiceStateUpdate()
				}
				LaunchedEffect(isShortcutsEnabled) {
					if (isShortcutsEnabled) {
						shortcutManager.addShortcuts()
					} else {
						shortcutManager.removeShortcuts()
					}
				}
			}

			LaunchedEffect(appState.backendMessage) {
				when (val message = appState.backendMessage) {
					is BackendMessage.Failure -> {
						when (message.exception) {
							is VpnException.InvalidCredential -> {
								if (NymVpn.isForeground()) {
									SnackbarController.showMessage(StringValue.StringResource(R.string.exception_cred_invalid))
									navController.go(Destination.Credential.route)
								}
							} else -> Unit
						}
					}
					else -> Unit
				}
			}

			fun getTheme(): Theme {
				return appState.settings.theme ?: theme?.let { Theme.valueOf(it) } ?: Theme.default()
			}

			showNavBar = when (Destination.from(navBackStackEntry?.destination?.route).title.asString(this)) {
				"" -> false
				else -> true
			}

			SnackbarControllerProvider { host ->
				NymVPNTheme(theme = getTheme()) {
					Scaffold(
						contentWindowInsets = WindowInsets(0.dp),
						modifier = Modifier.semantics {
							// Enables testTag -> UiAutomator resource id
							@OptIn(ExperimentalComposeUiApi::class)
							testTagsAsResourceId = true
						},
						topBar = {
							if (showNavBar) {
								NavBar(
									appState,
									navController,
									{ appViewModel.onNavBarTrailingClick() },
									Modifier
										.onGloballyPositioned {
											navHeight = with(density) {
												it.size.height.toDp()
											}
										},
								)
							}
						},
						snackbarHost = {
							SnackbarHost(host) { snackbarData: SnackbarData ->
								CustomSnackBar(message = snackbarData.visuals.message, paddingTop = navHeight)
							}
						},
					) { padding ->
						NavHost(
							navController,
							startDestination = if (isAnalyticsShown == true) Destination.Main.route else Destination.Analytics.route,
							modifier =
							Modifier
								.fillMaxSize()
								.padding(padding),
							enterTransition = { fadeIn(tween(200)) },
							exitTransition = { fadeOut(tween(200)) },
							popEnterTransition = { fadeIn(tween(200)) },
							popExitTransition = { fadeOut(tween(200)) },
						) {
							composable(
								Destination.Main.route,
							) {
								val autoStart = it.arguments?.getString("autoStart")
								MainScreen(navController, appState, autoStart.toBoolean())
							}
							composable(Destination.Analytics.route) { AnalyticsScreen(appViewModel, navController, appState) }
							composable(Destination.Permission.route) { nav ->
								val argument = nav.arguments?.getString("permission")
								requireNotNull(argument) { "No permission passed" }
								runCatching {
									val permission = Permission.valueOf(argument)
									PermissionScreen(navController, permission)
								}
							}
							composable(Destination.Settings.route) {
								SettingsScreen(
									appViewModel,
									navController,
									appState,
								)
							}
							composable(Destination.EntryLocation.route) {
								HopScreen(
									gatewayLocation = GatewayLocation.ENTRY,
									appViewModel,
									navController,
									appState,

								)
							}
							composable(Destination.ExitLocation.route) {
								HopScreen(
									gatewayLocation = GatewayLocation.EXIT,
									appViewModel,
									navController,
									appState,
								)
							}
							composable(Destination.Logs.route) { LogsScreen() }
							composable(Destination.Support.route) { SupportScreen() }
							composable(Destination.Feedback.route) { FeedbackScreen() }
							composable(Destination.Legal.route) { LegalScreen(navController) }
							composable(Destination.Credential.route) {
								CredentialScreen()
							}
							composable(Destination.Account.route) { AccountScreen(navController, appState) }
							composable(Destination.Licenses.route) {
								LicensesScreen()
							}
							composable(Destination.Appearance.route) {
								AppearanceScreen(navController)
							}
							composable(Destination.Display.route) {
								DisplayScreen(appState)
							}
							composable(Destination.Language.route) {
								LanguageScreen(navController, localeStorage)
							}
							composable(Destination.Environment.route) {
								EnvironmentScreen(appState)
							}
						}
					}
				}
			}
		}
	}

	override fun attachBaseContext(newBase: Context) {
		oldPrefLocaleCode = LocaleStorage(newBase).getPreferredLocale()
		applyOverrideConfiguration(LocaleUtil.getLocalizedConfiguration(oldPrefLocaleCode))
		super.attachBaseContext(newBase)
	}

	override fun onResume() {
		val currentLocaleCode = LocaleStorage(this).getPreferredLocale()
		if (oldPrefLocaleCode != currentLocaleCode) {
			recreate() // locale is changed, restart the activity to update
			oldPrefLocaleCode = currentLocaleCode
		}
		super.onResume()
	}
}
