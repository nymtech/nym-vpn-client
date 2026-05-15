package net.nymtech.nymvpn.ui.theme

import androidx.compose.runtime.Immutable
import androidx.compose.runtime.staticCompositionLocalOf
import androidx.compose.ui.graphics.Color

internal object DarkSchemeBaseColors {
	val primary = Color(0xFF59F09F)
	val onPrimary = Color(0xFF1C1B1F)
	val primaryContainer = Color(0xFF2E2E2E)
	val onPrimaryContainer = Color(0xFFFFFFFF)
	val secondary = Color(0xFFB0ADB6)
	val onSecondary = Color(0xFF1C1B1F)
	val tertiary = Color(0xFF76FFB1)
	val background = Color(0xFF090909)
	val onBackground = Color(0xFFAEACB1)
	val surface = Color(0xFF1D1D1F)
	val onSurface = Color(0xFFAEACB1)
	val surfaceContainer = Color(0xFF313033)
	val inverseSurface = Color(0xFFEEEEEE)
	val error = Color(0xFFFF4444)
	val errorContainer = Color(0xFFCD2C3C)
	val onErrorContainer = Color(0xFFFFFFFF)
	val outline = Color(0xFF66656A)
}

internal object LightSchemeBaseColors {
	val primary = Color(0xFF5BF0A0)
	val onPrimary = Color(0xFF1C1B1F)
	val primaryContainer = Color(0xFFFFFFFF)
	val onPrimaryContainer = Color(0xFF111111)
	val secondary = Color(0xFFB0ADB6)
	val onSecondary = Color(0xFF1C1B1F)
	val tertiary = Color(0xFF28C96C)
	val background = Color(0xFFD5D5D5)
	val onBackground = Color(0xFF8A8990)
	val surface = Color(0xFFFFFFFF)
	val onSurface = Color(0xFF6A7282)
	val surfaceContainer = Color(0xFFFFFFFF)
	val inverseSurface = Color(0xFFEEEEEE)
	val error = Color(0xFFFF4444)
	val errorContainer = Color(0xFFCD2C3C)
	val onErrorContainer = Color(0xFFFFFFFF)
	val outline = Color(0xFF66656A)
}

@Immutable
data class NymColors(
	val labelCyan: Color = Color.Unspecified,
	val iconCyan: Color = Color.Unspecified,
	val iconCyanBackground: Color = Color.Unspecified,
	val warning: Color = Color.Unspecified,
	val success: Color = Color.Unspecified,
	val iconBorder: Color = Color.Unspecified,
	val iconBackground: Color = Color.Unspecified,
	val statusConnectedBg: Color = Color.Unspecified,
	val borderCyan: Color = Color.Unspecified,
	val buttonErrorBorder: Color = Color.Unspecified,
	val buttonErrorText: Color = Color.Unspecified,
	val trackDefaultBackground: Color = Color.Unspecified,
	val navBarTitleBackground: Color = Color.Unspecified,
	val navBarIconTint: Color = Color.Unspecified,
	val switchBackground: Color = Color.Unspecified,
	val connectionFillColor: Color = Color.Unspecified,
)

val DarkNymColors = NymColors(
	labelCyan = Color(0xFF5BF0A0),
	iconCyan = Color(0xFF76FFB1),
	iconCyanBackground = Color(0xFF374042),
	success = Color(0xFF28C96C),
	warning = Color(0xFFFFCC33),
	iconBorder = Color(0x405BF0A0),
	iconBackground = Color(0x265BF0A0),
	statusConnectedBg = Color(0x1A5BF0A0),
	borderCyan = Color(0x8098DDFF),
	buttonErrorBorder = Color(0x99FF4444),
	buttonErrorText = Color(0xFFE73E14),
	trackDefaultBackground = Color(0x26FFFFFF),
	navBarTitleBackground = Color(0xFF090909),
	navBarIconTint = Color(0xFFAEACB1),
	switchBackground = Color(0xFF66656A),
	connectionFillColor = Color(0xFF5BF0A0),
)

val LightNymColors = NymColors(
	labelCyan = Color(0xFF28C96C),
	iconCyan = Color(0xFF28C96C),
	iconCyanBackground = Color(0xFFFFFFFF),
	success = Color(0xFF28C96C),
	warning = Color(0xFFFFCC33),
	iconBorder = Color(0x401A9B61),
	iconBackground = Color(0x261A9B61),
	statusConnectedBg = Color(0x1A1A9B61),
	borderCyan = Color(0x8000A3F5),
	buttonErrorBorder = Color(0x99FF4444),
	buttonErrorText = Color(0xFFE73E14),
	trackDefaultBackground = Color(0x1F0A0A0A),
	navBarTitleBackground = Color(0xFFFFFFFF),
	navBarIconTint = Color(0xFF111111),
	switchBackground = Color(0xFFD5D5D5),
	connectionFillColor = Color(0xFF1A9B61),
)

val LocalNymColors = staticCompositionLocalOf { DarkNymColors }
