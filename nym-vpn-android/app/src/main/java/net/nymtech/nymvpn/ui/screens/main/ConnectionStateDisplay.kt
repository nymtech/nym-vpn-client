package net.nymtech.nymvpn.ui.screens.main

import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.platform.LocalContext
import net.nymtech.nymvpn.ui.common.animations.Pulse
import net.nymtech.nymvpn.ui.common.labels.PillLabel
import net.nymtech.nymvpn.ui.model.ConnectionState
import net.nymtech.nymvpn.ui.theme.CustomColors

@Composable
fun ConnectionStateDisplay(connectionState: ConnectionState) {
	val context = LocalContext.current
	val text = connectionState.status.asString(context)
	when (connectionState) {
		ConnectionState.Connected ->
			PillLabel(
				text = text,
				backgroundColor = CustomColors.statusGreen,
				textColor = CustomColors.confirm,
			)

		ConnectionState.Disconnected ->
			PillLabel(
				text = text,
				backgroundColor =
				if (isSystemInDarkTheme()) {
					CustomColors.statusDefaultDark
				} else {
					CustomColors.statusDefaultLight
				},
				textColor = MaterialTheme.colorScheme.onSecondary,
			)
		is ConnectionState.Connecting ->
			PillLabel(
				text = text,
				backgroundColor =
				if (isSystemInDarkTheme()) {
					CustomColors.statusDefaultDark
				} else {
					CustomColors.statusDefaultLight
				},
				textColor = MaterialTheme.colorScheme.onBackground,
				trailing = { Pulse() },
			)

		ConnectionState.Disconnecting ->
			PillLabel(
				text = text,
				backgroundColor =
				if (isSystemInDarkTheme()) {
					CustomColors.statusDefaultDark
				} else {
					CustomColors.statusDefaultLight
				},
				textColor = MaterialTheme.colorScheme.onBackground,
				trailing = { Pulse() },
			)
	}
}
