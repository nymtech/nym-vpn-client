package net.nymtech.nymvpn.ui.screens.main.panel.components

import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.BorderStroke
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.common.buttons.MainStyledButton
import net.nymtech.nymvpn.ui.model.ConnectionState
import net.nymtech.nymvpn.ui.screens.main.panel.ConnectAction
import net.nymtech.nymvpn.ui.theme.CustomTypography
import net.nymtech.nymvpn.util.extensions.isVpnAlwaysOn
import net.nymtech.nymvpn.util.extensions.scaledHeight
import nym_vpn_lib_types.AccountControllerErrorStateReason
import nym_vpn_lib_types.AccountControllerState
import nym_vpn_lib_types.ErrorStateReason

@Composable
internal fun ActionButton(
	connectionState: ConnectionState,
	accountState: AccountControllerState,
	isMnemonicStored: Boolean,
	isSubscriptionExpired: Boolean,
	hasSubscriptionHistory: Boolean,
	onAction: (ConnectAction) -> Unit,
	modifier: Modifier = Modifier,
) {
	val context = LocalContext.current
	val buttonModifier = modifier.fillMaxWidth().height(48.dp.scaledHeight())
	val buttonShape = RoundedCornerShape(50)

	when (connectionState) {
		ConnectionState.Disconnected,
		ConnectionState.Offline,
		ConnectionState.WaitingForConnection,
		-> {
			val isAccountNotActive = accountState is AccountControllerState.Error &&
				accountState.v1 is AccountControllerErrorStateReason.AccountStatusNotActive
			val isPendingSubscription = accountState is AccountControllerState.PendingSubscription
			val label = when {
				isAccountNotActive -> R.string.connect
				isSubscriptionExpired -> if (hasSubscriptionHistory) R.string.error_expired_subscription_button else R.string.error_no_subscription_button
				isMnemonicStored -> R.string.connect
				else -> R.string.get_started
			}
			MainStyledButton(
				onClick = {
					onAction(
						when {
							isPendingSubscription -> ConnectAction.REFRESH_ACCOUNT
							!isAccountNotActive && isMnemonicStored && !isSubscriptionExpired -> ConnectAction.CONNECT
							else -> ConnectAction.GET_STARTED
						},
					)
				},
				content = { Text(stringResource(label), style = MaterialTheme.typography.titleMedium, color = MaterialTheme.colorScheme.onPrimary) },
				color = if (isPendingSubscription) MaterialTheme.colorScheme.secondary else MaterialTheme.colorScheme.primary,
				modifier = buttonModifier,
				shape = buttonShape,
			)
		}

		is ConnectionState.Connecting -> MainStyledButton(
			onClick = { onAction(ConnectAction.DISCONNECT) },
			content = { Text(stringResource(R.string.connecting), style = MaterialTheme.typography.titleMedium) },
			color = MaterialTheme.colorScheme.secondary,
			modifier = buttonModifier,
			shape = buttonShape,
		)

		ConnectionState.Disconnecting -> MainStyledButton(
			onClick = {},
			content = { Text(stringResource(R.string.disconnecting), style = MaterialTheme.typography.titleMedium) },
			color = MaterialTheme.colorScheme.secondary,
			modifier = buttonModifier,
			shape = buttonShape,
		)

		ConnectionState.Connected -> MainStyledButton(
			onClick = { onAction(ConnectAction.DISCONNECT) },
			textColor = MaterialTheme.colorScheme.onPrimaryContainer,
			content = { Text(stringResource(R.string.disconnect), style = MaterialTheme.typography.titleMedium) },
			color = Color.Transparent,
			borderStroke = BorderStroke(1.dp, MaterialTheme.colorScheme.outline),
			modifier = buttonModifier,
			shape = buttonShape,
		)

		is ConnectionState.Error -> {
			val isSubscriptionError = connectionState.reason is ErrorStateReason.InactiveSubscription ||
				connectionState.reason is ErrorStateReason.InactiveAccount
			val isAccountActionPending = accountState == AccountControllerState.Syncing ||
				accountState == AccountControllerState.PendingSubscription

			when {
				isSubscriptionError && !isAccountActionPending && isVpnAlwaysOn(context) -> MainStyledButton(
					onClick = { onAction(ConnectAction.STOP_KILL_SWITCH) },
					textColor = MaterialTheme.colorScheme.onError,
					content = { Text(stringResource(R.string.stop), style = CustomTypography.buttonMain) },
					color = MaterialTheme.colorScheme.error,
					modifier = buttonModifier,
					shape = buttonShape,
				)
				isSubscriptionError && !isAccountActionPending -> MainStyledButton(
					onClick = { onAction(ConnectAction.GET_STARTED) },
					content = { Text(stringResource(R.string.get_started), style = CustomTypography.buttonMain) },
					modifier = buttonModifier,
					shape = buttonShape,
				)
				else -> MainStyledButton(
					onClick = { onAction(if (isMnemonicStored) ConnectAction.CONNECT else ConnectAction.GET_STARTED) },
					content = { Text(stringResource(if (isMnemonicStored) R.string.connect else R.string.get_started), style = CustomTypography.buttonMain) },
					modifier = buttonModifier,
					shape = buttonShape,
				)
			}
		}

		is ConnectionState.StartFailure -> MainStyledButton(
			onClick = { onAction(ConnectAction.CONNECT) },
			content = { Text(stringResource(R.string.connect), style = CustomTypography.buttonMain) },
			modifier = buttonModifier,
			shape = buttonShape,
		)
	}
}
