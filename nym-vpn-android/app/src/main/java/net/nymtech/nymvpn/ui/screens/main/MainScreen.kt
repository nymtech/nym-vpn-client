package net.nymtech.nymvpn.ui.screens.main

import android.Manifest
import android.app.Activity.RESULT_OK
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
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.defaultMinSize
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Info
import androidx.compose.material.icons.outlined.Speed
import androidx.compose.material.icons.outlined.VisibilityOff
import androidx.compose.material.ripple.rememberRipple
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
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
import com.google.accompanist.permissions.shouldShowRationale
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.NymVpn
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.AppUiState
import net.nymtech.nymvpn.ui.AppViewModel
import net.nymtech.nymvpn.ui.NavItem
import net.nymtech.nymvpn.ui.common.Modal
import net.nymtech.nymvpn.ui.common.animations.SpinningIcon
import net.nymtech.nymvpn.ui.common.buttons.IconSurfaceButton
import net.nymtech.nymvpn.ui.common.buttons.MainStyledButton
import net.nymtech.nymvpn.ui.common.functions.countryIcon
import net.nymtech.nymvpn.ui.common.labels.GroupLabel
import net.nymtech.nymvpn.ui.common.labels.StatusInfoLabel
import net.nymtech.nymvpn.ui.common.textbox.CustomTextField
import net.nymtech.nymvpn.ui.model.ConnectionState
import net.nymtech.nymvpn.ui.model.StateMessage
import net.nymtech.nymvpn.ui.theme.CustomColors
import net.nymtech.nymvpn.ui.theme.CustomTypography
import net.nymtech.nymvpn.ui.theme.iconSize
import net.nymtech.nymvpn.util.Constants
import net.nymtech.nymvpn.util.exceptions.NymVpnExceptions
import net.nymtech.nymvpn.util.extensions.buildCountryNameString
import net.nymtech.nymvpn.util.extensions.openWebUrl
import net.nymtech.nymvpn.util.extensions.scaledHeight
import net.nymtech.nymvpn.util.extensions.scaledWidth
import net.nymtech.vpn.model.VpnMode
import java.time.Instant

@OptIn(ExperimentalPermissionsApi::class)
@Composable
fun MainScreen(navController: NavController, appViewModel: AppViewModel, appUiState: AppUiState, viewModel: MainViewModel = hiltViewModel()) {
	val uiState by viewModel.uiState.collectAsStateWithLifecycle()
	val context = LocalContext.current
	val scope = rememberCoroutineScope()

	val notificationPermissionState =
		if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
			rememberPermissionState(Manifest.permission.POST_NOTIFICATIONS)
		} else {
			null
		}

	var showDialog by remember { mutableStateOf(false) }

	fun onConnectWithPermission() {
		scope.launch {
			viewModel.onConnect().onFailure {
				appViewModel.showSnackbarMessage(context.getString(R.string.exception_cred_invalid))
				navController.navigate(NavItem.Settings.Credential.route)
			}
		}
	}

	val vpnActivityResultState =
		rememberLauncherForActivityResult(
			ActivityResultContracts.StartActivityForResult(),
			onResult = {
				val accepted = (it.resultCode == RESULT_OK)
				if (!accepted) {
					navController.navigate("${NavItem.Permission.route}/${NavItem.Permission.Path.VPN}")
				} else {
					onConnectWithPermission()
				}
			},
		)

	fun requestNotificationPermissions(): Result<Unit> {
		if (notificationPermissionState == null || notificationPermissionState.status.isGranted) return Result.success(Unit)
		if (!notificationPermissionState.status.isGranted && !notificationPermissionState.status.shouldShowRationale
		) {
			notificationPermissionState.launchPermissionRequest()
		} else if (!notificationPermissionState.status.isGranted && notificationPermissionState.status.shouldShowRationale) {
			navController.navigate("${NavItem.Permission.route}/${NavItem.Permission.Path.NOTIFICATION}")
		}
		return Result.failure(NymVpnExceptions.PermissionsNotGrantedException())
	}

	fun onConnect() {
		requestNotificationPermissions().onSuccess {
			val intent = VpnService.prepare(context)
			if (intent != null) {
				vpnActivityResultState.launch(
					intent,
				)
			} else {
				onConnectWithPermission()
			}
		}
	}

	Modal(show = showDialog, onDismiss = { showDialog = false }, title = {
		Text(
			text = stringResource(R.string.mode_selection),
			color = MaterialTheme.colorScheme.onSurface,
			style = CustomTypography.labelHuge,
		)
	}, text = {
		ModeModalBody(
			onClick = {
				context.openWebUrl(context.getString(R.string.mode_support_link))
			},
		)
	})

	LaunchedEffect(uiState.firstHopCounty, uiState.lastHopCountry, uiState.networkMode, uiState.connectionState) {
		NymVpn.requestTileServiceStateUpdate()
	}

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
		val firstHopName = context.buildCountryNameString(uiState.firstHopCounty)
		val lastHopName = context.buildCountryNameString(uiState.lastHopCountry)
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
				modifier = Modifier.padding(horizontal = 24.dp.scaledWidth()),
			) {
				Row(
					horizontalArrangement = Arrangement.SpaceBetween,
					verticalAlignment = Alignment.CenterVertically,
					modifier = Modifier
						.fillMaxWidth()
						.padding(bottom = 16.dp.scaledHeight()),
				) {
					GroupLabel(title = stringResource(R.string.select_mode))
					IconButton(onClick = {
						showDialog = true
					}, modifier = Modifier.size(iconSize)) {
						val icon = Icons.Outlined.Info
						Icon(icon, icon.name, tint = MaterialTheme.colorScheme.outline)
					}
				}
				Column(verticalArrangement = Arrangement.spacedBy(24.dp.scaledHeight(), Alignment.Bottom)) {
					IconSurfaceButton(
						leadingIcon = Icons.Outlined.VisibilityOff,
						title = stringResource(R.string.five_hop_mixnet),
						description = stringResource(R.string.five_hop_description),
						onClick = {
							if (uiState.connectionState == ConnectionState.Disconnected) {
								viewModel.onFiveHopSelected()
							} else {
								appViewModel.showSnackbarMessage(context.getString(R.string.disabled_while_connected))
							}
						},
						selected = uiState.networkMode == VpnMode.FIVE_HOP_MIXNET,
					)
					IconSurfaceButton(
						leadingIcon = Icons.Outlined.Speed,
						title = stringResource(R.string.two_hop_mixnet),
						description = stringResource(R.string.two_hop_description),
						onClick = {
							if (uiState.connectionState == ConnectionState.Disconnected) {
								viewModel.onTwoHopSelected()
							} else {
								appViewModel.showSnackbarMessage(context.getString(R.string.disabled_while_connected))
							}
						},
						selected = uiState.networkMode == VpnMode.TWO_HOP_MIXNET,
					)
				}
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
						singleLine = true,
						modifier = Modifier
							.fillMaxWidth()
							.height(60.dp.scaledHeight())
							.defaultMinSize(minHeight = 1.dp, minWidth = 1.dp)
							.clickable(
								remember { MutableInteractionSource() },
								indication = if (selectionEnabled) rememberRipple() else null,
							) {
								if (selectionEnabled) {
									navController.navigate(
										NavItem.Location.Entry.route,
									)
								} else {
									appViewModel.showSnackbarMessage(context.getString(R.string.disabled_while_connected))
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
					singleLine = true,
					modifier = Modifier
						.fillMaxWidth()
						.height(60.dp.scaledHeight())
						.defaultMinSize(minHeight = 1.dp, minWidth = 1.dp)
						.clickable(remember { MutableInteractionSource() }, indication = if (selectionEnabled) rememberRipple() else null) {
							if (selectionEnabled) {
								navController.navigate(
									NavItem.Location.Exit.route,
								)
							} else {
								appViewModel.showSnackbarMessage(context.getString(R.string.disabled_while_connected))
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
								scope.launch {
									if (appUiState.credentialExpiryTime != null &&
										appUiState.credentialExpiryTime.isAfter(Instant.now())
									) {
										onConnect()
									} else {
										navController.navigate(NavItem.Settings.Credential.route)
									}
								}
							},
							content = {
								Text(
									stringResource(id = R.string.connect),
									style = CustomTypography.labelHuge,
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
									style = CustomTypography.labelHuge,
								)
							},
							color = CustomColors.disconnect,
						)
				}
			}
		}
	}
}
