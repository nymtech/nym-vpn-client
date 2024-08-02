package net.nymtech.nymvpn.ui

import android.content.Context
import android.content.pm.PackageManager
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.annotation.Keep
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Scaffold
import androidx.compose.material3.SnackbarData
import androidx.compose.material3.SnackbarDuration
import androidx.compose.material3.SnackbarHost
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.SnackbarResult
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
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.lifecycleScope
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.rememberNavController
import com.zaneschepke.localizationutil.LocaleStorage
import com.zaneschepke.localizationutil.LocaleUtil
import dagger.hilt.android.AndroidEntryPoint
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.NymVpn
import net.nymtech.nymvpn.data.SettingsRepository
import net.nymtech.nymvpn.module.IoDispatcher
import net.nymtech.nymvpn.module.MainImmediateDispatcher
import net.nymtech.nymvpn.ui.common.labels.CustomSnackBar
import net.nymtech.nymvpn.ui.common.navigation.NavBar
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
import net.nymtech.nymvpn.ui.screens.settings.feedback.FeedbackScreen
import net.nymtech.nymvpn.ui.screens.settings.legal.LegalScreen
import net.nymtech.nymvpn.ui.screens.settings.legal.licenses.LicensesScreen
import net.nymtech.nymvpn.ui.screens.settings.logs.LogsScreen
import net.nymtech.nymvpn.ui.screens.settings.support.SupportScreen
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.util.Constants
import net.nymtech.nymvpn.util.StringValue
import timber.log.Timber
import java.util.Locale
import javax.inject.Inject

@AndroidEntryPoint
@Keep
class MainActivity : ComponentActivity() {

	private val localeStorage: LocaleStorage by lazy {
		(application as NymVpn).localeStorage
	}

	@Inject
	@MainImmediateDispatcher
	lateinit var mainImmediateDispatcher: CoroutineDispatcher

	@Inject
	@IoDispatcher
	lateinit var ioDispatcher: CoroutineDispatcher

	@Inject
	lateinit var settingsRepository: SettingsRepository

	private lateinit var oldPrefLocaleCode: String

	private fun resetTitle() {
		try {
			val label = packageManager.getActivityInfo(componentName, PackageManager.GET_META_DATA).labelRes
			if (label != 0) {
				setTitle(label)
			}
		} catch (e: PackageManager.NameNotFoundException) {
			Timber.e(e)
		}
	}

	override fun onCreate(savedInstanceState: Bundle?) {
		super.onCreate(savedInstanceState)

		resetTitle()

		val isAnalyticsShown = intent.extras?.getBoolean(SplashActivity.IS_ANALYTICS_SHOWN_INTENT_KEY)
		val theme = intent.extras?.getString(SplashActivity.THEME)

		setContent {
			val appViewModel = hiltViewModel<AppViewModel>()
			val uiState by appViewModel.uiState.collectAsStateWithLifecycle(lifecycle = this.lifecycle)

			val navController = rememberNavController()
			val snackbarHostState = remember { SnackbarHostState() }
			var navHeight by remember { mutableStateOf(0.dp) }
			val density = LocalDensity.current

			navController.addOnDestinationChangedListener { controller, destination, _ ->
				Timber.d("Destination: $destination")
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

			fun showSnackBarMessage(message: StringValue) {
				lifecycleScope.launch(mainImmediateDispatcher) {
					val result =
						snackbarHostState.showSnackbar(
							message = message.asString(this@MainActivity),
							duration = SnackbarDuration.Short,
						)
					when (result) {
						SnackbarResult.ActionPerformed,
						SnackbarResult.Dismissed,
						-> {
							snackbarHostState.currentSnackbarData?.dismiss()
						}
					}
				}
			}

			fun onNavBarTrailingClick() {
				navController.currentBackStackEntry?.destination?.route?.let {
					when (Destination.valueOf(it)) {
						Destination.Main -> navController.navigate(Destination.Settings.route)
						Destination.EntryLocation, Destination.ExitLocation -> appViewModel.onToggleShowLocationTooltip()
						else -> Unit
					}
				}
			}

			LaunchedEffect(uiState.snackbarMessageConsumed) {
				if (!uiState.snackbarMessageConsumed) {
					showSnackBarMessage(StringValue.DynamicString(uiState.snackbarMessage))
					appViewModel.snackbarMessageConsumed()
				}
			}

			fun getTheme(): Theme {
				return uiState.settings.theme ?: theme?.let { Theme.valueOf(it) } ?: Theme.default()
			}

			NymVPNTheme(theme = getTheme()) {
				Scaffold(
					Modifier.semantics {
						// Enables testTag -> UiAutomator resource id
						@OptIn(ExperimentalComposeUiApi::class)
						testTagsAsResourceId = true
					},
					topBar = {
						NavBar(
							uiState,
							navController,
							{ onNavBarTrailingClick() },
							Modifier
								.onGloballyPositioned {
									navHeight = with(density) {
										it.size.height.toDp()
									}
								},
						)
					},
					snackbarHost = {
						SnackbarHost(snackbarHostState) { snackbarData: SnackbarData ->
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
					) {
						composable(
							Destination.Main.route,
						) {
							val autoStart = it.arguments?.getString("autoStart")
							MainScreen(navController, appViewModel, uiState, autoStart.toBoolean())
						}
						composable(Destination.Analytics.route) { AnalyticsScreen(navController, appViewModel, uiState) }
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
								navController,
								appViewModel = appViewModel,
								uiState,
							)
						}
						composable(Destination.EntryLocation.route) {
							HopScreen(
								navController = navController,
								gatewayLocation = GatewayLocation.ENTRY,
								appViewModel,
							)
						}
						composable(Destination.ExitLocation.route) {
							HopScreen(
								navController = navController,
								gatewayLocation = GatewayLocation.EXIT,
								appViewModel,
							)
						}
						composable(Destination.Logs.route) { LogsScreen(appViewModel = appViewModel) }
						composable(Destination.Support.route) { SupportScreen() }
						composable(Destination.Feedback.route) { FeedbackScreen() }
						composable(Destination.Legal.route) { LegalScreen(navController) }
						composable(Destination.Credential.route) {
							CredentialScreen(
								navController,
								appViewModel,
							)
						}
						composable(Destination.Account.route) { AccountScreen(appViewModel, uiState, navController) }
						composable(Destination.Licenses.route) {
							LicensesScreen(
								appViewModel,
							)
						}
						composable(Destination.Appearance.route) {
							AppearanceScreen(navController)
						}
						composable(Destination.Display.route) {
							DisplayScreen()
						}
						composable(Destination.Language.route) {
							LanguageScreen(navController, localeStorage)
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
			recreate() // locale is changed, restart the activty to update
			oldPrefLocaleCode = currentLocaleCode
		}
		super.onResume()
	}
}
