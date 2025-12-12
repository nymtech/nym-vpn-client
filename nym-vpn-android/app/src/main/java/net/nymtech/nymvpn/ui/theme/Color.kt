package net.nymtech.nymvpn.ui.theme

import androidx.compose.runtime.Immutable
import androidx.compose.ui.graphics.Color

sealed class ThemeColors(
	val background: Color,
	val surface: Color,
	val primary: Color,
	val secondary: Color,
	val onBackground: Color,
	val onSurface: Color,
	val onPrimary: Color,
	val onSurfaceVariant: Color,
	val onSecondary: Color,
	val surfaceContainer: Color,
	val tertiary: Color,
	val outline: Color,
	val error: Color,
) {
	data object Dark : ThemeColors(
		background = Color(0xFF242B2D),
		surface = Color(0xFF374042),
		primary = primary,
		secondary = secondary,
		onBackground = Color(0xFFFFFFFF),
		onSurface = Color(0xFFE6E1E5),
		onPrimary = Color(0xFF242B2D),
		onSurfaceVariant = Color(0xFF938F99),
		onSecondary = Color(0xFF56545A),
		surfaceContainer = Color(0xFF313033),
		tertiary = Color(0xFF14E76F),
		outline = Color(0xFFB0ADB6),
		error = Color(0xFFE33B5A),
	)

	data object Light : ThemeColors(
		background = Color(0xFFEBEEF4),
		surface = Color(0xFFFFFFFF),
		primary = primary,
		secondary = secondary,
		onBackground = Color(0xFF1C1B1F),
		onSurface = Color(0xFF1C1B1F),
		onPrimary = Color(0xFF1C1B1F),
		onSurfaceVariant = Color(0xFF79747E),
		onSecondary = Color(0xFFA4A4A4),
		surfaceContainer = Color(0xFFFFFFFF),
		tertiary = Color(0xFF0B8A42),
		outline = Color(0xFF606060),
		error = Color(0xFFE33B5A),
	)
}

val primary = Color(0xFF14E76F)
val secondary = Color(0XFFCECCD1)

object CustomColors {
	val outlineVariant = Color(0xFF49454F)
	val statusGreen = Color(0x1A47C45D)
	val statusRed = Color(0xFF672D32)
	val statusRedLight = Color(0xFFF3CAC8)
	val statusDefaultDark = Color(0xFF2B3234)
	val statusDefaultLight = Color(0xFFE2E4EA)
	val errorStatusPillLight = Color(0xFFEBEEF4)
	val errorStatusPillDark = Color(0xFF242B2D)
	val pulse = Color(0xFF7075FF)
	val disconnect = Color(0xFFE02C4D)
	val error = Color(0xFFE33B5A)
	val snackBarBackgroundColor = Color(0xFF484649)
	val snackbarTextColor = Color(0xFFE7E7E7)
	val warning = Color(0xFFFFB400)
	val iconBorder = Color(0x4014E76F)
	val iconBackground = Color(0x2611C55F)
	val buttonRedTransparent = Color(0x1AED5060)
	val buttonRedTransparentBorder = Color(0xFFED5060)
}

@Immutable
data class CustomColorsPalette(
	val borderCyan: Color = Color.Unspecified,
	val labelCyan: Color = Color.Unspecified,
	val redIconBackground: Color = Color.Unspecified,
	val redIcon: Color = Color.Unspecified,
	val greyIconBackground: Color = Color.Unspecified,
	val greyIcon: Color = Color.Unspecified,
	val greenIconBackground: Color = Color.Unspecified,
	val greenIcon: Color = Color.Unspecified,
)

val LightCustomColorsPalette = CustomColorsPalette(
	borderCyan = Color(0x8000A3F5),
	labelCyan = Color(0xFF00A3F5),
	redIconBackground = Color(0xFFFFE2E2),
	redIcon = Color(0xFFE73E14),
	greyIconBackground = Color(0xFFE2E4EA),
	greyIcon = Color(0xFFB0ADB6),
	greenIconBackground = Color(0xFFAFFFD2),
	greenIcon = Color(0xFF0B8A42),
)

val DarkCustomColorsPalette = CustomColorsPalette(
	borderCyan = Color(0x8098DDFF),
	labelCyan = Color(0xFF98DDFF),
	redIconBackground = Color(0xFF500E0E),
	redIcon = Color(0xFFE73E14),
	greyIconBackground = Color(0xFF32393B),
	greyIcon = Color(0xFF66646C),
	greenIconBackground = Color(0xFF254B35),
	greenIcon = Color(0xFF14E76F),
)
