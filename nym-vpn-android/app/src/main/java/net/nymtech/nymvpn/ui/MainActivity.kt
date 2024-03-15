package net.nymtech.nymvpn.ui
import android.Manifest
import android.os.Build
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
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
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.core.splashscreen.SplashScreen.Companion.installSplashScreen
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.lifecycleScope
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.rememberNavController
import com.google.accompanist.permissions.ExperimentalPermissionsApi
import com.google.accompanist.permissions.isGranted
import com.google.accompanist.permissions.rememberPermissionState
import dagger.hilt.android.AndroidEntryPoint
import io.sentry.android.core.SentryAndroid
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.BuildConfig
import net.nymtech.nymvpn.data.datastore.DataStoreManager
import net.nymtech.nymvpn.ui.common.labels.CustomSnackBar
import net.nymtech.nymvpn.ui.common.navigation.NavBar
import net.nymtech.nymvpn.ui.screens.hop.HopScreen
import net.nymtech.nymvpn.ui.screens.main.MainScreen
import net.nymtech.nymvpn.ui.screens.settings.SettingsScreen
import net.nymtech.nymvpn.ui.screens.settings.account.AccountScreen
import net.nymtech.nymvpn.ui.screens.settings.display.DisplayScreen
import net.nymtech.nymvpn.ui.screens.settings.feedback.FeedbackScreen
import net.nymtech.nymvpn.ui.screens.settings.legal.LegalScreen
import net.nymtech.nymvpn.ui.screens.settings.legal.licenses.LicensesScreen
import net.nymtech.nymvpn.ui.screens.settings.login.LoginScreen
import net.nymtech.nymvpn.ui.screens.settings.logs.LogsScreen
import net.nymtech.nymvpn.ui.screens.settings.support.SupportScreen
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.TransparentSystemBars
import net.nymtech.nymvpn.util.Constants
import net.nymtech.nymvpn.util.StringValue
import net.nymtech.vpn.NymVpnClient
import net.nymtech.vpn.NymVpnService
import net.nymtech.vpn.model.Hop
import net.nymtech.vpn.model.VpnMode
import javax.inject.Inject

@AndroidEntryPoint
class MainActivity : ComponentActivity() {

  @Inject lateinit var dataStoreManager: DataStoreManager

  @OptIn(ExperimentalPermissionsApi::class)
  override fun onCreate(savedInstanceState: Bundle?) {
    super.onCreate(savedInstanceState)

    installSplashScreen()

    lifecycleScope.launch {
      //init data
      dataStoreManager.init()
      //setup sentry
      val reportingEnabled = dataStoreManager.getFromStore(DataStoreManager.ERROR_REPORTING)
      if(reportingEnabled ?: BuildConfig.OPT_IN_REPORTING) {
        if(reportingEnabled == null) dataStoreManager.saveToDataStore(DataStoreManager.ERROR_REPORTING, true)
        SentryAndroid.init(this@MainActivity) { options ->
          options.enableTracing = true
          options.enableAllAutoBreadcrumbs(true)
          options.isEnableUserInteractionTracing = true
          options.isEnableUserInteractionBreadcrumbs = true
          options.dsn = BuildConfig.SENTRY_DSN
          options.sampleRate = 1.0
          options.tracesSampleRate = 1.0
          options.profilesSampleRate = 1.0
          options.environment = if(BuildConfig.DEBUG) Constants.SENTRY_DEV_ENV else Constants.SENTRY_PROD_ENV
        }
      }
    }

    setContent {

      val appViewModel = hiltViewModel<AppViewModel>()
      val uiState by appViewModel.uiState.collectAsStateWithLifecycle()
      val navController = rememberNavController()
      val snackbarHostState = remember { SnackbarHostState() }

      val notificationPermissionState = if(Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU)
        rememberPermissionState(Manifest.permission.POST_NOTIFICATIONS) else null

      fun requestNotificationPermission() {
        if (notificationPermissionState != null && !notificationPermissionState.status.isGranted
        ) {
          notificationPermissionState.launchPermissionRequest()
        }
      }

      LaunchedEffect(Unit) {
        appViewModel.updateCountryListCache()
        appViewModel.readLogCatOutput()
        requestNotificationPermission()
      }

      fun showSnackBarMessage(message: StringValue) {
        lifecycleScope.launch(Dispatchers.Main) {
          val result =
            snackbarHostState.showSnackbar(
              message = message.asString(this@MainActivity),
              duration = SnackbarDuration.Short,
            )
          when (result) {
            SnackbarResult.ActionPerformed,
            SnackbarResult.Dismissed -> {
              snackbarHostState.currentSnackbarData?.dismiss()
            }
          }
        }
      }

      LaunchedEffect(uiState.snackbarMessageConsumed) {
        if(!uiState.snackbarMessageConsumed) {
          showSnackBarMessage(StringValue.DynamicString(uiState.snackbarMessage))
          appViewModel.snackbarMessageConsumed()
        }
      }

      NymVPNTheme(theme = uiState.theme) {
        // A surface container using the 'background' color from the theme
        TransparentSystemBars()
        Scaffold(
            topBar = { NavBar(appViewModel,navController) },
            snackbarHost = {
              SnackbarHost(snackbarHostState) { snackbarData: SnackbarData ->
                CustomSnackBar(message = snackbarData.visuals.message)
              }
            }
        ) {

            NavHost(navController, startDestination = NavItem.Main.route,
              modifier = Modifier
                .fillMaxSize()
                .padding(it)) {
              composable(NavItem.Main.route) { MainScreen(navController, uiState) }
              composable(NavItem.Settings.route) { SettingsScreen(navController, uiState) }
              composable(NavItem.Hop.Entry.route) {
                appViewModel.updateCountryListCache()
                HopScreen(navController =  navController, hopType =  HopType.FIRST)
              }
              composable(NavItem.Hop.Exit.route) {
                appViewModel.updateCountryListCache()
                HopScreen(navController =  navController, hopType = HopType.LAST)
              }
              composable(NavItem.Settings.Display.route) { DisplayScreen() }
              composable(NavItem.Settings.Logs.route) { LogsScreen(appViewModel) }
              composable(NavItem.Settings.Support.route) { SupportScreen(appViewModel) }
              composable(NavItem.Settings.Feedback.route) { FeedbackScreen(appViewModel) }
              composable(NavItem.Settings.Legal.route) { LegalScreen(appViewModel,navController) }
              composable(NavItem.Settings.Login.route) { LoginScreen(navController, appViewModel) }
              composable(NavItem.Settings.Account.route) { AccountScreen(appViewModel) }
              composable(NavItem.Settings.Legal.Licenses.route){ LicensesScreen(appViewModel) }
            }
          }
        }
      }
    }
  }

