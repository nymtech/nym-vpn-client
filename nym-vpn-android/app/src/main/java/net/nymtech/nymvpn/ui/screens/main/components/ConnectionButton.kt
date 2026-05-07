package net.nymtech.nymvpn.ui.screens.main.components

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import androidx.navigation.NavController
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.Route
import net.nymtech.nymvpn.ui.common.buttons.MainStyledButton
import net.nymtech.nymvpn.ui.common.snackbar.AlertType
import net.nymtech.nymvpn.ui.common.snackbar.NymAlertController
import net.nymtech.nymvpn.ui.common.snackbar.NymAlertMessage
import net.nymtech.nymvpn.ui.model.ConnectionState
import net.nymtech.nymvpn.ui.theme.CustomTypography
import net.nymtech.nymvpn.util.Constants
import net.nymtech.nymvpn.util.extensions.goFromRoot
import net.nymtech.nymvpn.util.extensions.isVpnAlwaysOn
import net.nymtech.nymvpn.util.extensions.scaledHeight
import net.nymtech.nymvpn.util.extensions.scaledWidth
import nym_vpn_lib_types.AccountControllerState
import nym_vpn_lib_types.ErrorStateReason

@Composable
fun ConnectionButton(
	connectionState: ConnectionState,
	accountState: AccountControllerState,
	isMnemonicStored: Boolean,
	navController: NavController,
	onConnect: () -> Unit,
	onDisconnect: () -> Unit,
	onStopKillSwitch: () -> Unit,
	onGetStart: () -> Unit,
	modifier: Modifier = Modifier,
) {
	val context = LocalContext.current
	val noInternetText = stringResource(R.string.no_internet)
	val buttonModifier = Modifier.fillMaxWidth().height(56.dp.scaledHeight())

	Box(modifier = modifier.padding(horizontal = 24.dp.scaledWidth())) {
		when (connectionState) {
			ConnectionState.Disconnected,
			ConnectionState.Offline,
			ConnectionState.WaitingForConnection,
			-> MainStyledButton(
				testTag = Constants.CONNECT_TEST_TAG,
				onClick = {
					when {
						!isMnemonicStored -> navController.goFromRoot(Route.Welcome)
						connectionState is ConnectionState.Offline -> NymAlertController.show(
							NymAlertMessage(type = AlertType.Neutral, title = noInternetText),
						)
						else -> onConnect()
					}
				},
				content = {
					Text(
						stringResource(if (isMnemonicStored) R.string.connect else R.string.get_started),
						style = CustomTypography.buttonMain,
					)
				},
				modifier = buttonModifier,
			)

			is ConnectionState.Error -> {
				val isSubscriptionError = connectionState.reason is ErrorStateReason.InactiveSubscription ||
					connectionState.reason is ErrorStateReason.InactiveAccount

				if (isSubscriptionError && accountState != AccountControllerState.Syncing && accountState != AccountControllerState.PendingSubscription) {
					if (isVpnAlwaysOn(context)) {
						MainStyledButton(
							onClick = onStopKillSwitch,
							content = {
								Text(
									stringResource(R.string.stop),
									style = CustomTypography.buttonMain,
									color = MaterialTheme.colorScheme.background,
								)
							},
							color = MaterialTheme.colorScheme.error,
							modifier = buttonModifier,
						)
					} else {
						MainStyledButton(
							onClick = onGetStart,
							content = {
								Text(
									stringResource(R.string.get_started),
									style = CustomTypography.buttonMain,
								)
							},
							modifier = buttonModifier,
						)
					}
				} else {
					MainStyledButton(
						onClick = {
							if (!isMnemonicStored) navController.goFromRoot(Route.Welcome)
							else onConnect()
						},
						content = {
							Text(
								stringResource(R.string.connect),
								style = CustomTypography.buttonMain,
							)
						},
						modifier = buttonModifier,
					)
				}
			}

			ConnectionState.Disconnecting,
			is ConnectionState.Connecting,
			-> MainStyledButton(
				onClick = onDisconnect,
				content = {
					Text(
						stringResource(R.string.stop),
						style = CustomTypography.buttonMain,
						color = MaterialTheme.colorScheme.background,
					)
				},
				color = MaterialTheme.colorScheme.error,
				modifier = buttonModifier,
			)

			is ConnectionState.StartFailure -> MainStyledButton(
				onClick = onConnect,
				content = {
					Text(stringResource(R.string.connect), style = CustomTypography.buttonMain)
				},
				modifier = buttonModifier,
			)

			ConnectionState.Connected -> MainStyledButton(
				testTag = Constants.DISCONNECT_TEST_TAG,
				onClick = onDisconnect,
				content = {
					Text(
						stringResource(R.string.disconnect),
						style = CustomTypography.buttonMain,
					)
				},
				color = MaterialTheme.colorScheme.error,
				modifier = buttonModifier,
			)
		}
	}
}
