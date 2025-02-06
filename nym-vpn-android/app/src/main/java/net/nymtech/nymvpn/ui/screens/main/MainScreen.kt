package net.nymtech.nymvpn.ui.screens.main

import android.app.Activity.RESULT_OK
import android.net.VpnService
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.foundation.Image
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.asPaddingValues
import androidx.compose.foundation.layout.defaultMinSize
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.systemBars
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Info
import androidx.compose.material.icons.outlined.Settings
import androidx.compose.material.icons.outlined.Speed
import androidx.compose.material.icons.outlined.VisibilityOff
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.SnackbarHost
import androidx.compose.material3.SnackbarHostState
import androidx.compose.material3.Text
import androidx.compose.material3.ripple
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
import kotlinx.coroutines.delay
import kotlinx.coroutines.launch
import net.nymtech.connectivity.NetworkStatus
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.manager.backend.model.BackendUiEvent
import net.nymtech.nymvpn.ui.AppUiState
import net.nymtech.nymvpn.ui.AppViewModel
import net.nymtech.nymvpn.ui.Route
import net.nymtech.nymvpn.ui.common.Modal
import net.nymtech.nymvpn.ui.common.buttons.IconSurfaceButton
import net.nymtech.nymvpn.ui.common.buttons.MainStyledButton
import net.nymtech.nymvpn.ui.common.labels.GroupLabel
import net.nymtech.nymvpn.ui.common.labels.StatusInfoLabel
import net.nymtech.nymvpn.ui.common.navigation.LocalNavController
import net.nymtech.nymvpn.ui.common.navigation.MainTitle
import net.nymtech.nymvpn.ui.common.navigation.NavBarState
import net.nymtech.nymvpn.ui.common.navigation.NavIcon
import net.nymtech.nymvpn.ui.common.snackbar.SnackbarController
import net.nymtech.nymvpn.ui.common.textbox.CustomTextField
import net.nymtech.nymvpn.ui.model.ConnectionState
import net.nymtech.nymvpn.ui.model.StateMessage
import net.nymtech.nymvpn.ui.model.StateMessage.Error
import net.nymtech.nymvpn.ui.model.StateMessage.StartError
import net.nymtech.nymvpn.ui.screens.permission.Permission
import net.nymtech.nymvpn.ui.theme.CustomColors
import net.nymtech.nymvpn.ui.theme.CustomTypography
import net.nymtech.nymvpn.ui.theme.iconSize
import net.nymtech.nymvpn.util.Constants
import net.nymtech.nymvpn.util.extensions.convertSecondsToTimeString
import net.nymtech.nymvpn.util.extensions.getFlagImageVectorByName
import net.nymtech.nymvpn.util.extensions.goFromRoot
import net.nymtech.nymvpn.util.extensions.openWebUrl
import net.nymtech.nymvpn.util.extensions.scaledHeight
import net.nymtech.nymvpn.util.extensions.scaledWidth
import net.nymtech.nymvpn.util.extensions.toUserMessage
import net.nymtech.vpn.backend.Tunnel

@Composable
fun MainScreen(appViewModel: AppViewModel, appUiState: AppUiState, autoStart: Boolean, viewModel: MainViewModel = hiltViewModel()) {
	val uiState = remember(appUiState) {
		with(appUiState) {
			val connectionState = when {
				managerState.tunnelState != Tunnel.State.Down && networkStatus == NetworkStatus.Disconnected -> ConnectionState.WaitingForConnection
				managerState.tunnelState == Tunnel.State.Down && networkStatus == NetworkStatus.Disconnected -> ConnectionState.Offline
				else -> ConnectionState.from(managerState.tunnelState)
			}
			val stateMessage = when (val event = managerState.backendUiEvent) {
				is BackendUiEvent.BandwidthAlert, null -> connectionState.stateMessage
				is BackendUiEvent.Failure -> Error(event.reason)
				is BackendUiEvent.StartFailure -> StartError(event.exception)
			}
			MainUiState(
				connectionTime = managerState.connectionData?.connectedAt,
				connectionState = connectionState,
				stateMessage = stateMessage,
			)
		}
	}

	val context = LocalContext.current
	val snackbar = SnackbarController.current
	val screenSnackbar = remember { SnackbarHostState() }
	val scope = rememberCoroutineScope()
	val padding = WindowInsets.systemBars.asPaddingValues()
	val navController = LocalNavController.current

	var didAutoStart by remember { mutableStateOf(false) }
	var showDialog by remember { mutableStateOf(false) }
	var connectionTime: String? by remember { mutableStateOf(null) }

	LaunchedEffect(Unit) {
		appViewModel.onNavBarStateChange(
			NavBarState(
				title = { MainTitle() },
				trailing = {
					NavIcon(Icons.Outlined.Settings) {
						navController.goFromRoot(Route.Settings)
					}
				},
			),
		)
	}

	with(appUiState.managerState) {
		LaunchedEffect(tunnelState) {
			while (tunnelState == Tunnel.State.Up && connectionData != null) {
				connectionData.connectedAt?.let {
					connectionTime = (System.currentTimeMillis() / 1000L - it).convertSecondsToTimeString()
					delay(1000)
				}
			}
			connectionTime = null
		}
	}

	fun whenDisconnected(callback: () -> Unit) {
		when (uiState.connectionState) {
			ConnectionState.Disconnected, ConnectionState.Offline -> callback.invoke()
			ConnectionState.WaitingForConnection, is ConnectionState.Connecting -> snackbar.showMessage(context.getString(R.string.disabled_while_connecting))
			else -> snackbar.showMessage(context.getString(R.string.disabled_while_connected))
		}
	}

	val vpnActivityResultState =
		rememberLauncherForActivityResult(
			ActivityResultContracts.StartActivityForResult(),
			onResult = {
				val accepted = (it.resultCode == RESULT_OK)
				if (!accepted) {
					navController.goFromRoot(Route.Permission(Permission.VPN))
				} else {
					viewModel.onConnect()
				}
			},
		)

	fun onConnectPressed() {
		val intent = VpnService.prepare(context)
		if (intent != null) {
			vpnActivityResultState.launch(
				intent,
			)
		} else {
			viewModel.onConnect()
		}
	}

	if (autoStart && !didAutoStart) {
		LaunchedEffect(Unit) {
			didAutoStart = true
			onConnectPressed()
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

	Column(
		verticalArrangement = Arrangement.spacedBy(24.dp.scaledHeight(), Alignment.Top),
		horizontalAlignment = Alignment.CenterHorizontally,
		modifier = Modifier.verticalScroll(rememberScrollState()).fillMaxSize().padding(bottom = padding.calculateBottomPadding()),
	) {
		Column(
			verticalArrangement = Arrangement.spacedBy(8.dp.scaledHeight()),
			horizontalAlignment = Alignment.CenterHorizontally,
			modifier = Modifier.padding(top = 68.dp.scaledHeight()),
		) {
			SnackbarHost(hostState = screenSnackbar, Modifier)
			ConnectionStateDisplay(connectionState = uiState.connectionState)
			uiState.stateMessage.let {
				when (it) {
					is StateMessage.Status ->
						StatusInfoLabel(
							message = it.message.asString(context),
							textColor = MaterialTheme.colorScheme.onSurfaceVariant,
						)

					is Error ->
						StatusInfoLabel(
							message = it.reason.toUserMessage(context),
							textColor = CustomColors.error,
						)

					is StartError -> {
						StatusInfoLabel(
							message = it.exception.toUserMessage(context),
							textColor = CustomColors.error,
						)
					}
				}
			}
			AnimatedVisibility(visible = connectionTime != null) {
				connectionTime?.let {
					StatusInfoLabel(
						message = it,
						textColor = MaterialTheme.colorScheme.onSurface,
					)
				}
			}
		}
		Spacer(modifier = Modifier.weight(1f))
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
						leadingIcon = Icons.Outlined.Speed,
						title = stringResource(R.string.two_hop_title),
						description = stringResource(R.string.two_hop_description),
						onClick = {
							whenDisconnected {
								viewModel.onTwoHopSelected()
							}
						},
						selected = appUiState.settings.vpnMode == Tunnel.Mode.TWO_HOP_MIXNET,
					)
					IconSurfaceButton(
						leadingIcon = Icons.Outlined.VisibilityOff,
						title = stringResource(R.string.five_hop_mixnet),
						description = stringResource(R.string.five_hop_description),
						onClick = {
							whenDisconnected {
								viewModel.onFiveHopSelected()
							}
						},
						selected = appUiState.settings.vpnMode == Tunnel.Mode.FIVE_HOP_MIXNET,
					)
				}
			}
			Column(
				verticalArrangement = Arrangement.spacedBy(24.dp.scaledHeight(), Alignment.Bottom),
				modifier = Modifier.padding(horizontal = 24.dp.scaledWidth()),
			) {
				GroupLabel(title = stringResource(R.string.connect_to))
				val trailingIcon = ImageVector.vectorResource(R.drawable.link_arrow_right)
				val indication = when (uiState.connectionState) {
					ConnectionState.Disconnected, ConnectionState.Offline -> ripple()
					else -> null
				}
				CustomTextField(
					value = appUiState.entryPointName,
					readOnly = true,
					enabled = false,
					label = {
						Text(
							stringResource(R.string.first_hop),
							style = MaterialTheme.typography.bodySmall,
						)
					},
					leading = {
						val image = ImageVector.vectorResource(context.getFlagImageVectorByName(appUiState.entryPointCountry))
						Image(
							image,
							image.name,
							modifier =
							Modifier
								.padding(horizontal = 16.dp.scaledWidth(), vertical = 16.dp.scaledHeight())
								.size(
									iconSize,
								),
						)
					},

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
							indication = indication,
						) {
							whenDisconnected {
								navController.goFromRoot(
									Route.EntryLocation,
								)
							}
						},
				)
				CustomTextField(
					value = appUiState.exitPointName,
					readOnly = true,
					enabled = false,
					label = {
						Text(
							stringResource(R.string.last_hop),
							style = MaterialTheme.typography.bodySmall,
						)
					},
					leading = {
						val image = ImageVector.vectorResource(context.getFlagImageVectorByName(appUiState.exitPointCountry))
						Image(
							image,
							image.name,
							modifier =
							Modifier
								.padding(horizontal = 16.dp.scaledWidth(), vertical = 16.dp.scaledHeight())
								.size(
									iconSize,
								),
						)
					},
					trailing = {
						Icon(trailingIcon, trailingIcon.name, tint = MaterialTheme.colorScheme.onSurface)
					},
					singleLine = true,
					modifier = Modifier
						.fillMaxWidth()
						.height(60.dp.scaledHeight())
						.defaultMinSize(minHeight = 1.dp, minWidth = 1.dp)
						.clickable(remember { MutableInteractionSource() }, indication = indication) {
							whenDisconnected {
								navController.goFromRoot(
									Route.ExitLocation,
								)
							}
						},
				)
			}
			Box(modifier = Modifier.padding(horizontal = 24.dp.scaledWidth())) {
				when (uiState.connectionState) {
					is ConnectionState.Disconnected, ConnectionState.Offline ->
						MainStyledButton(
							testTag = Constants.CONNECT_TEST_TAG,
							onClick = {
								scope.launch {
									if (!appUiState.managerState.isMnemonicStored) return@launch navController.goFromRoot(Route.Credential)
									if (uiState.connectionState is ConnectionState.Offline) return@launch snackbar.showMessage(context.getString(R.string.no_internet))
									onConnectPressed()
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
					is ConnectionState.WaitingForConnection,
					-> {
						MainStyledButton(
							onClick = {
								viewModel.onDisconnect()
							},
							content = {
								Text(
									stringResource(id = R.string.stop),
									style = CustomTypography.labelHuge,
									color = MaterialTheme.colorScheme.background,
								)
							},
							color = MaterialTheme.colorScheme.secondary,
						)
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
