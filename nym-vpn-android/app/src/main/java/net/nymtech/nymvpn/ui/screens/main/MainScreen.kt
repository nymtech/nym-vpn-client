package net.nymtech.nymvpn.ui.screens.main

import android.Manifest
import android.net.VpnService
import android.os.Build
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.defaultMinSize
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.heightIn
import androidx.compose.foundation.layout.padding
import androidx.compose.material.ripple.rememberRipple
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
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
import com.google.accompanist.permissions.ExperimentalPermissionsApi
import com.google.accompanist.permissions.isGranted
import com.google.accompanist.permissions.rememberPermissionState
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.AppUiState
import net.nymtech.nymvpn.ui.AppViewModel
import net.nymtech.nymvpn.ui.NavItem
import net.nymtech.nymvpn.ui.common.animations.SpinningIcon
import net.nymtech.nymvpn.ui.common.buttons.MainStyledButton
import net.nymtech.nymvpn.ui.common.buttons.RadioSurfaceButton
import net.nymtech.nymvpn.ui.common.functions.countryIcon
import net.nymtech.nymvpn.ui.common.labels.GroupLabel
import net.nymtech.nymvpn.ui.common.labels.StatusInfoLabel
import net.nymtech.nymvpn.ui.common.textbox.CustomTextField
import net.nymtech.nymvpn.ui.model.ConnectionState
import net.nymtech.nymvpn.ui.model.StateMessage
import net.nymtech.nymvpn.ui.theme.CustomColors
import net.nymtech.nymvpn.util.Constants
import net.nymtech.nymvpn.util.StringUtils
import net.nymtech.nymvpn.util.scaledHeight
import net.nymtech.nymvpn.util.scaledWidth
import net.nymtech.vpn.model.VpnMode

@OptIn(ExperimentalPermissionsApi::class)
@Composable
fun MainScreen(navController: NavController, appViewModel: AppViewModel, appUiState: AppUiState, viewModel: MainViewModel = hiltViewModel()) {
	val uiState by viewModel.uiState.collectAsStateWithLifecycle()
	val context = LocalContext.current

	val notificationPermissionState =
		if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
			rememberPermissionState(Manifest.permission.POST_NOTIFICATIONS)
		} else {
			null
		}

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
		modifier = Modifier.fillMaxSize(),
	) {
		Column(
			verticalArrangement = Arrangement.spacedBy(8.dp.scaledHeight()),
			horizontalAlignment = Alignment.CenterHorizontally,
			modifier = Modifier.padding(top = 68.dp.scaledHeight()),
		) {
			ConnectionStateDisplay(connectionState = uiState.connectionState)
			uiState.stateMessage.let {
				when (it) {
					is StateMessage.Info ->
						StatusInfoLabel(
							message = it.message.asString(context),
							textColor = MaterialTheme.colorScheme.onSurfaceVariant,
						)

					is StateMessage.Error ->
						StatusInfoLabel(
							message = it.message.asString(context),
							textColor = CustomColors.error,
						)
				}
			}
			AnimatedVisibility(visible = uiState.connectionTime != "") {
				StatusInfoLabel(
					message = uiState.connectionTime,
					textColor = MaterialTheme.colorScheme.onSurface,
				)
			}
		}
		val firstHopName = StringUtils.buildCountryNameString(uiState.firstHopCounty, context)
		val lastHopName = StringUtils.buildCountryNameString(uiState.lastHopCountry, context)
		val firstHopIcon = countryIcon(uiState.firstHopCounty)
		val lastHopIcon = countryIcon(uiState.lastHopCountry)
		Column(
			verticalArrangement = Arrangement.spacedBy(36.dp.scaledHeight(), Alignment.Bottom),
			horizontalAlignment = Alignment.CenterHorizontally,
			modifier =
			Modifier
				.fillMaxSize()
				.padding(bottom = 24.dp.scaledHeight()),
		) {
			Column(
				verticalArrangement = Arrangement.spacedBy(24.dp.scaledHeight(), Alignment.Bottom),
				modifier = Modifier.padding(horizontal = 24.dp.scaledWidth()),
			) {
				GroupLabel(title = stringResource(R.string.select_network))
				RadioSurfaceButton(
					leadingIcon = ImageVector.vectorResource(R.drawable.mixnet),
					title = stringResource(R.string.five_hop_mixnet),
					description = stringResource(R.string.five_hop_description),
					onClick = {
						if (uiState.connectionState == ConnectionState.Disconnected) {
							viewModel.onFiveHopSelected()
						}
					},
					selected = uiState.networkMode == VpnMode.FIVE_HOP_MIXNET,
				)
				RadioSurfaceButton(
					leadingIcon = ImageVector.vectorResource(R.drawable.shield),
					title = stringResource(R.string.two_hop_mixnet),
					description = stringResource(R.string.two_hop_description),
					onClick = {
						if (uiState.connectionState == ConnectionState.Disconnected) {
							viewModel.onTwoHopSelected()
						}
					},
					selected = uiState.networkMode == VpnMode.TWO_HOP_MIXNET,
				)
			}
			Column(
				verticalArrangement = Arrangement.spacedBy(24.dp.scaledHeight(), Alignment.Bottom),
				modifier = Modifier.padding(horizontal = 24.dp.scaledWidth()),
			) {
				GroupLabel(title = stringResource(R.string.connect_to))
				val trailingIcon = ImageVector.vectorResource(R.drawable.link_arrow_right)
				val selectionEnabled = uiState.connectionState is ConnectionState.Disconnected
				if (uiState.firstHopEnabled) {
					CustomTextField(
						value = firstHopName,
						readOnly = true,
						enabled = false,
						label = {
							Text(
								stringResource(R.string.first_hop),
								style = MaterialTheme.typography.bodySmall,
							)
						},
						leading = firstHopIcon,
						trailing = {
							Icon(trailingIcon, trailingIcon.name, tint = MaterialTheme.colorScheme.onSurface)
						},
						modifier = Modifier
							.fillMaxWidth()
							.heightIn(60.dp.scaledHeight())
							.defaultMinSize(minHeight = 1.dp, minWidth = 1.dp)
							.clickable(
								remember { MutableInteractionSource() },
								indication = if (selectionEnabled) rememberRipple() else null,
							) {
								if (selectionEnabled) {
									navController.navigate(
										NavItem.Hop.Entry.route,
									)
								}
							},
					)
				}
				CustomTextField(
					value = lastHopName,
					readOnly = true,
					enabled = false,
					label = {
						Text(
							stringResource(R.string.last_hop),
							style = MaterialTheme.typography.bodySmall,
						)
					},
					leading = lastHopIcon,
					trailing = {
						Icon(trailingIcon, trailingIcon.name, tint = MaterialTheme.colorScheme.onSurface)
					},
					modifier = Modifier
						.fillMaxWidth()
						.heightIn(60.dp.scaledHeight())
						.defaultMinSize(minHeight = 1.dp, minWidth = 1.dp)
						.clickable(remember { MutableInteractionSource() }, indication = if (selectionEnabled) rememberRipple() else null) {
							if (selectionEnabled) {
								navController.navigate(
									NavItem.Hop.Exit.route,
								)
							}
						},
				)
			}
			Box(modifier = Modifier.padding(horizontal = 24.dp.scaledWidth())) {
				when (uiState.connectionState) {
					is ConnectionState.Disconnected ->
						MainStyledButton(
							testTag = Constants.CONNECT_TEST_TAG,
							onClick = {
								if (viewModel.isCredentialImported()) {
									if (notificationPermissionState != null &&
										!notificationPermissionState.status.isGranted
									) {
										return@MainStyledButton notificationPermissionState.launchPermissionRequest()
									}
									if (vpnIntent != null) {
										return@MainStyledButton vpnActivityResultState.launch(
											vpnIntent,
										)
									}
									if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.UPSIDE_DOWN_CAKE && !appViewModel.isAlarmPermissionGranted()) {
										return@MainStyledButton appViewModel.requestAlarmPermission()
									}
									viewModel.onConnect()
								} else {
									navController.navigate(NavItem.Settings.Login.route)
								}
							},
							content = {
								Text(
									stringResource(id = R.string.connect),
									style = MaterialTheme.typography.labelLarge,
								)
							},
						)

					is ConnectionState.Disconnecting,
					is ConnectionState.Connecting,
					-> {
						val loading = ImageVector.vectorResource(R.drawable.loading)
						MainStyledButton(onClick = {}, content = { SpinningIcon(icon = loading) })
					}

					is ConnectionState.Connected ->
						MainStyledButton(
							testTag = Constants.DISCONNECT_TEST_TAG,
							onClick = { viewModel.onDisconnect() },
							content = {
								Text(
									stringResource(id = R.string.disconnect),
									style = MaterialTheme.typography.labelLarge,
								)
							},
							color = CustomColors.disconnect,
						)
				}
			}
		}
	}
}
