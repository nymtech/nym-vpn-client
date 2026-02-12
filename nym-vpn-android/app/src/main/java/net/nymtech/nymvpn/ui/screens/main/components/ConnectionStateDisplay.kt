package net.nymtech.nymvpn.ui.screens.main.components

import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.stringResource
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.common.animations.Pulse
import net.nymtech.nymvpn.ui.common.labels.PillLabel
import net.nymtech.nymvpn.ui.model.ConnectionState
import net.nymtech.nymvpn.ui.theme.CustomColors
import net.nymtech.nymvpn.ui.theme.Theme
import nym_vpn_lib_types.ErrorStateReason

@Composable
fun ConnectionStateDisplay(connectionState: ConnectionState, theme: Theme) {
	val isDarkMode = isSystemInDarkTheme()

	fun determinePillColor(lightColor: Color, darkColor: Color): Color {
		return when (theme) {
			Theme.AUTOMATIC, Theme.DYNAMIC -> if (isDarkMode) darkColor else lightColor
			Theme.DARK_MODE -> darkColor
			Theme.LIGHT_MODE -> lightColor
		}
	}

	when (connectionState) {
		ConnectionState.Connected ->
			PillLabel(
				text = stringResource(R.string.connected),
				backgroundColor = CustomColors.statusGreen,
				textColor = MaterialTheme.colorScheme.tertiary,
			)

		ConnectionState.Disconnected -> {
			PillLabel(
				text = stringResource(R.string.disconnected),
				backgroundColor = determinePillColor(CustomColors.statusDefaultLight, CustomColors.statusDefaultDark),
				textColor = MaterialTheme.colorScheme.onSecondary,
			)
		}

		is ConnectionState.Connecting ->
			PillLabel(
				text = stringResource(R.string.connecting),
				backgroundColor = determinePillColor(CustomColors.statusDefaultLight, CustomColors.statusDefaultDark),
				textColor = MaterialTheme.colorScheme.onBackground,
				trailing = { Pulse() },
			)

		ConnectionState.Disconnecting ->
			PillLabel(
				text = stringResource(R.string.disconnecting),
				backgroundColor = determinePillColor(CustomColors.statusDefaultLight, CustomColors.statusDefaultDark),
				textColor = MaterialTheme.colorScheme.onBackground,
				trailing = { Pulse() },
			)

		ConnectionState.Offline -> PillLabel(
			text = stringResource(R.string.offline),
			backgroundColor = determinePillColor(CustomColors.statusRedLight, CustomColors.statusRed),
			textColor = MaterialTheme.colorScheme.onSurface,
		)

		ConnectionState.WaitingForConnection -> PillLabel(
			text = stringResource(R.string.waiting_for_connection),
			backgroundColor = determinePillColor(CustomColors.statusDefaultLight, CustomColors.statusDefaultDark),
			textColor = MaterialTheme.colorScheme.onBackground,
			trailing = { Pulse(color = CustomColors.error) },
		)

		is ConnectionState.Error -> {
			val isSubscriptionError = connectionState.reason == ErrorStateReason.InactiveSubscription

			if (isSubscriptionError) {
				PillLabel(
					text = stringResource(R.string.pill_subscription_expired),
					backgroundColor = CustomColors.error,
					textColor = determinePillColor(CustomColors.errorStatusPillLight, CustomColors.errorStatusPillDark),
				)
			} else {
				PillLabel(
					text = stringResource(R.string.pill_error),
					backgroundColor = CustomColors.error,
					textColor = determinePillColor(CustomColors.errorStatusPillLight, CustomColors.errorStatusPillDark),
				)
			}
		}

		is ConnectionState.StartFailure -> {
			PillLabel(
				text = stringResource(R.string.pill_error),
				backgroundColor = CustomColors.error,
				textColor = determinePillColor(CustomColors.errorStatusPillLight, CustomColors.errorStatusPillDark),
			)
		}
	}
}
