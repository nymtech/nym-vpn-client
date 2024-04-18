package net.nymtech.nymvpn.ui.theme

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
) {
	data object Dark : ThemeColors(
		background = Color(0xFF1C1B1F),
		surface = Color(0x14D0BCFF),
		primary = primary,
		secondary = secondary,
		onBackground = Color(0xFFFFFFFF),
		onSurface = Color(0xFFE6E1E5),
		onPrimary = Color(0xFF1C1B1F),
		onSurfaceVariant = Color(0xFF938F99),
		onSecondary = Color(0xFF56545A),
	)

	data object Light : ThemeColors(
		background = Color(0xFFF2F4F6),
		surface = Color(0xFFFFFFFF),
		primary = primary,
		secondary = secondary,
		onBackground = Color(0xFF1C1B1F),
		onSurface = Color(0xFF1C1B1F),
		onPrimary = Color(0xFFFFFFFF),
		onSurfaceVariant = Color(0xFF79747E),
		onSecondary = Color(0xFFA4A4A4),
	)
}

val primary = Color(0xFFFB6E4E)
val secondary = Color(0XFF625B71)

object CustomColors {
	val outlineVariant = Color(0xFF49454F)
	val confirm = Color(0xFF2BC761)
	val statusGreen = Color(0x1A47C45D)
	val statusDefaultDark = Color(0xFF313033).copy(alpha = 0.16f)
	val statusDefaultLight = Color(0xFF625B71).copy(alpha = 0.16f)
	val disconnect = Color(0xFF7075FF)
	val error = Color(0xFFE33B5A)
	val snackBarBackgroundColor = Color(0xFF484649)
	val snackbarTextColor = Color(0xFFE7E7E7)
}
