package net.nymtech.nymvpn.ui

import android.os.Bundle
import androidx.activity.ComponentActivity
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.compose.setContent
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Scaffold
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.lifecycleScope
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.rememberNavController
import androidx.window.core.layout.WindowHeightSizeClass
import androidx.window.core.layout.WindowSizeClass
import androidx.window.layout.WindowMetricsCalculator
import dagger.hilt.android.AndroidEntryPoint
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.data.datastore.DataStoreManager
import net.nymtech.nymvpn.model.Country
import net.nymtech.nymvpn.ui.common.navigation.NavBar
import net.nymtech.nymvpn.ui.screens.hop.HopScreen
import net.nymtech.nymvpn.ui.screens.main.MainScreen
import net.nymtech.nymvpn.ui.screens.settings.SettingsScreen
import net.nymtech.nymvpn.ui.screens.settings.display.DisplayScreen
import net.nymtech.nymvpn.ui.screens.settings.feedback.FeedbackScreen
import net.nymtech.nymvpn.ui.screens.settings.feedback.review.ReviewScreen
import net.nymtech.nymvpn.ui.screens.settings.legal.LegalScreen
import net.nymtech.nymvpn.ui.screens.settings.logs.LogsScreen
import net.nymtech.nymvpn.ui.screens.settings.support.SupportScreen
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.TransparentSystemBars
import net.nymtech.vpn_client.NymVpnClient
import javax.inject.Inject

@AndroidEntryPoint
class MainActivity : ComponentActivity() {

  @Inject lateinit var dataStoreManager: DataStoreManager

  override fun onCreate(savedInstanceState: Bundle?) {
    super.onCreate(savedInstanceState)
    // load into memory, init data here
    val countries = listOf(
      Country("DE", "Germany", true),
      Country("DE", "Germany"),
      Country("FR", "France"),
      Country("US", "United States")
    )

    //determine window height
    val metrics = WindowMetricsCalculator.getOrCreate().computeCurrentWindowMetrics(this)
    val width = metrics.bounds.width()
    val height = metrics.bounds.height()
    val density = resources.displayMetrics.density
    val windowSize = WindowSizeClass.compute(width/density, height/density)
    windowHeightSizeClass = windowSize.windowHeightSizeClass

    lifecycleScope.launch {
      dataStoreManager.init()
      dataStoreManager.saveToDataStore(DataStoreManager.NODE_COUNTRIES, countries.toString())
    }

    setContent {

      //TODO refactor when vpn permission request is called
      var vpnIntent by remember { mutableStateOf(NymVpnClient().prepare(this)) }
      val vpnActivityResultState =
        rememberLauncherForActivityResult(
          ActivityResultContracts.StartActivityForResult(),
          onResult = {
            val accepted = (it.resultCode == RESULT_OK)
            if (accepted) {
              vpnIntent = null
            }
          },
        )
      LaunchedEffect(vpnIntent) {
        if (vpnIntent != null) {
          vpnActivityResultState.launch(vpnIntent)
        }
      }


      val mainViewModel = hiltViewModel<AppViewModel>()
      val uiState by mainViewModel.uiState.collectAsStateWithLifecycle()
      val navController = rememberNavController()
      NymVPNTheme(theme = uiState.theme) {
        // A surface container using the 'background' color from the theme
        TransparentSystemBars()
        Scaffold(
            topBar = { NavBar(navController) },
        ) {
          Column(modifier = Modifier.padding(it)) {
            NavHost(navController, startDestination = NavItem.Main.route) {
              composable(NavItem.Main.route) { MainScreen(navController) }
              composable(NavItem.Settings.route) { SettingsScreen(navController) }
              composable(NavItem.Hop.Entry.route) {
                HopScreen(navController =  navController, hopType =  HopType.FIRST)
              }
              composable(NavItem.Hop.Exit.route) {
                HopScreen(navController =  navController, hopType = HopType.LAST)
              }
              composable(NavItem.Settings.Display.route) { DisplayScreen() }
              composable(NavItem.Settings.Logs.route) { LogsScreen() }
              composable(NavItem.Settings.Support.route) { SupportScreen() }
              composable(NavItem.Settings.Feedback.route) { FeedbackScreen(navController) }
              composable(NavItem.Settings.Feedback.Review.route) { ReviewScreen() }
              composable(NavItem.Settings.Legal.route) { LegalScreen() }
            }
          }
        }
      }
    }
  }
  companion object {
    lateinit var windowHeightSizeClass: WindowHeightSizeClass
      private set
  }
}

