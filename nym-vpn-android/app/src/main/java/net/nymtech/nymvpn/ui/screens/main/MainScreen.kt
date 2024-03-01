package net.nymtech.nymvpn.ui.screens.main

import android.net.VpnService
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.res.vectorResource
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.navigation.NavController
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.model.NetworkMode
import net.nymtech.nymvpn.ui.AppUiState
import net.nymtech.nymvpn.ui.NavItem
import net.nymtech.nymvpn.ui.common.animations.SpinningIcon
import net.nymtech.nymvpn.ui.common.buttons.ListOptionSelectionButton
import net.nymtech.nymvpn.ui.common.buttons.MainStyledButton
import net.nymtech.nymvpn.ui.common.buttons.RadioSurfaceButton
import net.nymtech.nymvpn.ui.common.labels.GroupLabel
import net.nymtech.nymvpn.ui.common.labels.StatusInfoLabel
import net.nymtech.nymvpn.ui.common.labels.countryIcon
import net.nymtech.nymvpn.ui.model.ConnectionState
import net.nymtech.nymvpn.ui.model.StateMessage
import net.nymtech.nymvpn.ui.theme.CustomColors
import net.nymtech.nymvpn.util.StringUtils
import net.nymtech.nymvpn.util.scaledHeight
import net.nymtech.nymvpn.util.scaledWidth

@Composable
fun MainScreen(navController: NavController, appUiState: AppUiState, viewModel: MainViewModel = hiltViewModel()) {

  val uiState by viewModel.uiState.collectAsStateWithLifecycle()
  val context = LocalContext.current

    var vpnIntent by rememberSaveable { mutableStateOf(VpnService.prepare(context)) }
    val vpnActivityResultState =
        rememberLauncherForActivityResult(
            ActivityResultContracts.StartActivityForResult(),
            onResult = {
                vpnIntent = null
            },
        )


  Column(
      verticalArrangement = Arrangement.spacedBy(24.dp.scaledHeight(), Alignment.Top),
      horizontalAlignment = Alignment.CenterHorizontally,
      modifier = Modifier.fillMaxSize()) {
        Column(
            verticalArrangement = Arrangement.spacedBy(8.dp.scaledHeight()),
            horizontalAlignment = Alignment.CenterHorizontally, modifier = Modifier.padding(top = 68.dp.scaledHeight())) {
              ConnectionStateDisplay(connectionState = uiState.connectionState)
              uiState.stateMessage.let {
                when (it) {
                  is StateMessage.Info ->
                      StatusInfoLabel(
                          message = it.message.asString(context),
                          textColor = MaterialTheme.colorScheme.onSurfaceVariant)
                  is StateMessage.Error ->
                      StatusInfoLabel(
                          message = it.message.asString(context), textColor = CustomColors.error)
                }
              }
              StatusInfoLabel(
                  message = uiState.connectionTime, textColor = MaterialTheme.colorScheme.onSurface)
            }

        val firstHopName = StringUtils.buildCountryNameString(uiState.firstHopCounty, context)
        val lastHopName = StringUtils.buildCountryNameString(uiState.lastHopCountry, context)
        val firstHopIcon = countryIcon(uiState.firstHopCounty)
        val lastHopIcon = countryIcon(uiState.lastHopCountry)
        Column(
            verticalArrangement = Arrangement.spacedBy(36.dp.scaledHeight(), Alignment.Bottom),
            horizontalAlignment = Alignment.CenterHorizontally,
            modifier = Modifier
                .fillMaxSize()
                .padding(bottom = 24.dp.scaledHeight())) {
                Column(
                  verticalArrangement = Arrangement.spacedBy(24.dp.scaledHeight(), Alignment.Bottom),
                  modifier = Modifier.padding(horizontal = 24.dp.scaledWidth())) {
                    GroupLabel(title = stringResource(R.string.select_network))
                    RadioSurfaceButton(
                        leadingIcon = ImageVector.vectorResource(R.drawable.mixnet),
                        title = stringResource(R.string.five_hop),
                        description = stringResource(R.string.five_hop_description),
                        onClick = {
                          if (uiState.connectionState == ConnectionState.Disconnected)
                              viewModel.onFiveHopSelected()
                        },
                        selected = uiState.networkMode == NetworkMode.FIVE_HOP_MIXNET)
                    RadioSurfaceButton(
                        leadingIcon = ImageVector.vectorResource(R.drawable.shield),
                        title = stringResource(R.string.two_hop),
                        description = stringResource(R.string.two_hop_description),
                        onClick = {
                          if (uiState.connectionState == ConnectionState.Disconnected)
                              viewModel.onTwoHopSelected()
                        },
                        selected = uiState.networkMode == NetworkMode.TWO_HOP_WIREGUARD)
                  }
              Column(
                  verticalArrangement = Arrangement.spacedBy(24.dp.scaledHeight(), Alignment.Bottom),
                  modifier = Modifier.padding(horizontal = 24.dp.scaledWidth())) {
                    GroupLabel(title = stringResource(R.string.connect_to))
                    if (uiState.firstHopEnabled) {
                      ListOptionSelectionButton(
                          label = stringResource(R.string.first_hop),
                          value = firstHopName,
                          onClick = { navController.navigate(NavItem.Hop.Entry.route) },
                          leadingIcon = firstHopIcon)
                    }
                    ListOptionSelectionButton(
                        label = stringResource(R.string.last_hop),
                        value = lastHopName,
                        onClick = { navController.navigate(NavItem.Hop.Exit.route) },
                        leadingIcon = lastHopIcon)
                  }
              Box(modifier = Modifier.padding(horizontal = 24.dp.scaledWidth())) {
                when (uiState.connectionState) {
                  is ConnectionState.Disconnected ->
                      MainStyledButton(
                          onClick = {
                              if(appUiState.loggedIn) {
                                  if(vpnIntent != null)
                                  vpnActivityResultState.launch(vpnIntent)
                                  else viewModel.onConnect()
                              } else
                                  navController.navigate(NavItem.Settings.Login.route) },
                          content = {
                            Text(
                                stringResource(id = R.string.connect),
                                style = MaterialTheme.typography.labelLarge)
                          })
                  is ConnectionState.Disconnecting,
                  ConnectionState.Connecting -> {
                    val loading = ImageVector.vectorResource(R.drawable.loading)
                    MainStyledButton(onClick = {}, content = { SpinningIcon(icon = loading) })
                  }
                  is ConnectionState.Connected ->
                      MainStyledButton(
                          onClick = { viewModel.onDisconnect() },
                          content = {
                            Text(
                                stringResource(id = R.string.disconnect),
                                style = MaterialTheme.typography.labelLarge)
                          },
                          color = CustomColors.disconnect)
                }
              }
            }
      }
}
