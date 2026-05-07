package net.nymtech.nymvpn.ui.screens.main.components

import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.common.animations.Pulse
import net.nymtech.nymvpn.ui.common.labels.PillLabel
import net.nymtech.nymvpn.ui.model.ConnectionState
import net.nymtech.nymvpn.ui.theme.LocalNymColors
import nym_vpn_lib_types.ErrorStateReason

@Composable
fun ConnectionStateDisplay(connectionState: ConnectionState) {
	val colors = LocalNymColors.current

	when (connectionState) {
		ConnectionState.Connected -> PillLabel(
			text = stringResource(R.string.connected),
			backgroundColor = colors.statusConnectedBg,
			textColor = MaterialTheme.colorScheme.tertiary,
		)

		ConnectionState.Disconnected -> PillLabel(
			text = stringResource(R.string.disconnected),
			backgroundColor = MaterialTheme.colorScheme.surfaceVariant,
			textColor = MaterialTheme.colorScheme.onSecondary,
		)

		is ConnectionState.Connecting -> PillLabel(
			text = stringResource(R.string.connecting),
			backgroundColor = MaterialTheme.colorScheme.surfaceVariant,
			textColor = MaterialTheme.colorScheme.onBackground,
			trailing = { Pulse() },
		)

		ConnectionState.Disconnecting -> PillLabel(
			text = stringResource(R.string.disconnecting),
			backgroundColor = MaterialTheme.colorScheme.surfaceVariant,
			textColor = MaterialTheme.colorScheme.onBackground,
			trailing = { Pulse() },
		)

		ConnectionState.Offline -> PillLabel(
			text = stringResource(R.string.offline),
			backgroundColor = MaterialTheme.colorScheme.errorContainer,
			textColor = MaterialTheme.colorScheme.onSurface,
		)

		ConnectionState.WaitingForConnection -> PillLabel(
			text = stringResource(R.string.waiting_for_connection),
			backgroundColor = MaterialTheme.colorScheme.surfaceVariant,
			textColor = MaterialTheme.colorScheme.onBackground,
			trailing = { Pulse(color = MaterialTheme.colorScheme.error) },
		)

		is ConnectionState.Error -> {
			val isSubscriptionError = connectionState.reason == ErrorStateReason.InactiveSubscription
			PillLabel(
				text = stringResource(if (isSubscriptionError) R.string.pill_subscription_expired else R.string.pill_error),
				backgroundColor = MaterialTheme.colorScheme.error,
				textColor = MaterialTheme.colorScheme.background,
			)
		}

		is ConnectionState.StartFailure -> PillLabel(
			text = stringResource(R.string.pill_error),
			backgroundColor = MaterialTheme.colorScheme.error,
			textColor = MaterialTheme.colorScheme.background,
		)
	}
}
