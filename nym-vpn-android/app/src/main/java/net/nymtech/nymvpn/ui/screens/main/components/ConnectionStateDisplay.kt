package net.nymtech.nymvpn.ui.screens.main.components

import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalContext
import net.nymtech.nymvpn.ui.common.animations.Pulse
import net.nymtech.nymvpn.ui.common.labels.PillLabel
import net.nymtech.nymvpn.ui.model.ConnectionState
import net.nymtech.nymvpn.ui.theme.CustomColors
import net.nymtech.nymvpn.ui.theme.Theme

@Composable
fun ConnectionStateDisplay(connectionState: ConnectionState, theme: Theme) {
	val context = LocalContext.current
	val text = connectionState.status.asString(context)

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
				text = text,
				backgroundColor = CustomColors.statusGreen,
				textColor = MaterialTheme.colorScheme.tertiary,
			)

		ConnectionState.Disconnected ->
			PillLabel(
				text = text,
				backgroundColor = determinePillColor(CustomColors.statusDefaultLight, CustomColors.statusDefaultDark),
				textColor = MaterialTheme.colorScheme.onSecondary,
			)
		is ConnectionState.Connecting ->
			PillLabel(
				text = text,
				backgroundColor = determinePillColor(CustomColors.statusDefaultLight, CustomColors.statusDefaultDark),
				textColor = MaterialTheme.colorScheme.onBackground,
				trailing = { Pulse() },
			)

		ConnectionState.Disconnecting ->
			PillLabel(
				text = text,
				backgroundColor = determinePillColor(CustomColors.statusDefaultLight, CustomColors.statusDefaultDark),
				textColor = MaterialTheme.colorScheme.onBackground,
				trailing = { Pulse() },
			)

		ConnectionState.Offline -> PillLabel(
			text = text,
			backgroundColor = determinePillColor(CustomColors.statusRedLight, CustomColors.statusRed),
			textColor = MaterialTheme.colorScheme.onSurface,
		)
		ConnectionState.WaitingForConnection -> PillLabel(
			text = text,
			backgroundColor = determinePillColor(CustomColors.statusDefaultLight, CustomColors.statusDefaultDark),
			textColor = MaterialTheme.colorScheme.onBackground,
			trailing = { Pulse(color = CustomColors.error) },
		)
	}
}
