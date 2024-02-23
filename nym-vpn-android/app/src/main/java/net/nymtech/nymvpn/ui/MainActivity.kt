package net.nymtech.nymvpn.ui
import android.Manifest
import android.net.VpnService
import android.os.Build
import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.compose.setContent
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.layout.Column
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
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.data.datastore.DataStoreManager
import net.nymtech.nymvpn.model.Country
import net.nymtech.nymvpn.ui.common.labels.CustomSnackBar
import net.nymtech.nymvpn.ui.common.navigation.NavBar
import net.nymtech.nymvpn.ui.screens.hop.HopScreen
import net.nymtech.nymvpn.ui.screens.main.MainScreen
import net.nymtech.nymvpn.ui.screens.settings.SettingsScreen
import net.nymtech.nymvpn.ui.screens.settings.account.AccountScreen
import net.nymtech.nymvpn.ui.screens.settings.display.DisplayScreen
import net.nymtech.nymvpn.ui.screens.settings.feedback.FeedbackScreen
import net.nymtech.nymvpn.ui.screens.settings.legal.LegalScreen
import net.nymtech.nymvpn.ui.screens.settings.login.LoginScreen
import net.nymtech.nymvpn.ui.screens.settings.logs.LogsScreen
import net.nymtech.nymvpn.ui.screens.settings.support.SupportScreen
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.TransparentSystemBars
import net.nymtech.nymvpn.util.StringValue
import javax.inject.Inject

@AndroidEntryPoint
class MainActivity : ComponentActivity() {

  @Inject lateinit var dataStoreManager: DataStoreManager

  @OptIn(ExperimentalPermissionsApi::class)
  override fun onCreate(savedInstanceState: Bundle?) {
    super.onCreate(savedInstanceState)

    installSplashScreen()

    lifecycleScope.launch {
      dataStoreManager.init()
    }

    setContent {

      val mainViewModel = hiltViewModel<AppViewModel>()
      val uiState by mainViewModel.uiState.collectAsStateWithLifecycle()
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
        requestNotificationPermission()
      }

      LaunchedEffect(Unit) {
        mainViewModel.updateCountryListCache()
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

      NymVPNTheme(theme = uiState.theme) {
        // A surface container using the 'background' color from the theme
        TransparentSystemBars()
        Scaffold(
            topBar = { NavBar(navController) },
            snackbarHost = {
              SnackbarHost(snackbarHostState) { snackbarData: SnackbarData ->
                CustomSnackBar(message = snackbarData.visuals.message)
              }
            }
        ) {
          Column(modifier = Modifier.padding(it)) {
            NavHost(navController, startDestination = NavItem.Main.route) {
              composable(NavItem.Main.route) { MainScreen(navController, uiState) }
              composable(NavItem.Settings.route) { SettingsScreen(navController, uiState) }
              composable(NavItem.Hop.Entry.route) {
                mainViewModel.updateCountryListCache()
                HopScreen(navController =  navController, hopType =  HopType.FIRST)
              }
              composable(NavItem.Hop.Exit.route) {
                mainViewModel.updateCountryListCache()
                HopScreen(navController =  navController, hopType = HopType.LAST)
              }
              composable(NavItem.Settings.Display.route) { DisplayScreen() }
              composable(NavItem.Settings.Logs.route) { LogsScreen() }
              composable(NavItem.Settings.Support.route) { SupportScreen() }
              composable(NavItem.Settings.Feedback.route) { FeedbackScreen() }
              composable(NavItem.Settings.Legal.route) { LegalScreen() }
              composable(NavItem.Settings.Login.route) { LoginScreen(navController, showSnackbarMessage = { message -> showSnackBarMessage(message) } ) }
              composable(NavItem.Settings.Account.route) { AccountScreen() }
            }
          }
        }
      }
    }
  }
}

