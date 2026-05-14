package net.nymtech.nymvpn.ui.theme

import androidx.compose.material3.Typography
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.sp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.util.extensions.scaled

private val sans = FontFamily(Font(R.font.lab_grotesque_regular))
private val monoFont = FontFamily(Font(R.font.lab_grotesque_mono))

// ── Material 3 type scale ─────────────────────────────────────────────────────

val Typography = Typography(
	displayLarge = TextStyle(
		fontFamily = sans,
		fontWeight = FontWeight.Normal,
		fontSize = 57.sp.scaled(),
		lineHeight = 64.sp.scaled(),
		letterSpacing = (-0.25).sp.scaled(),
	),
	displayMedium = TextStyle(
		fontFamily = sans,
		fontWeight = FontWeight.Normal,
		fontSize = 45.sp.scaled(),
		lineHeight = 52.sp.scaled(),
		letterSpacing = 0.sp,
	),
	displaySmall = TextStyle(
		fontFamily = sans,
		fontWeight = FontWeight.Normal,
		fontSize = 36.sp.scaled(),
		lineHeight = 44.sp.scaled(),
		letterSpacing = 0.sp,
	),
	headlineLarge = TextStyle(
		fontFamily = sans,
		fontWeight = FontWeight.Normal,
		fontSize = 32.sp.scaled(),
		lineHeight = 40.sp.scaled(),
		letterSpacing = 0.sp,
	),
	headlineMedium = TextStyle(
		fontFamily = sans,
		fontWeight = FontWeight.Normal,
		fontSize = 28.sp.scaled(),
		lineHeight = 36.sp.scaled(),
		letterSpacing = 0.sp,
	),
	headlineSmall = TextStyle(
		fontFamily = sans,
		fontWeight = FontWeight.Normal,
		fontSize = 24.sp.scaled(),
		lineHeight = 32.sp.scaled(),
		letterSpacing = 0.sp,
		textAlign = TextAlign.Center,
	),
	titleLarge = TextStyle(
		fontFamily = sans,
		fontWeight = FontWeight.Medium,
		fontSize = 18.sp.scaled(),
		lineHeight = 24.sp.scaled(),
		letterSpacing = 0.sp,
	),
	titleMedium = TextStyle(
		fontFamily = sans,
		fontWeight = FontWeight.Medium,
		fontSize = 16.sp.scaled(),
		lineHeight = 24.sp.scaled(),
		letterSpacing = 0.15.sp.scaled(),
	),
	titleSmall = TextStyle(
		fontFamily = sans,
		fontWeight = FontWeight.Medium,
		fontSize = 14.sp.scaled(),
		lineHeight = 20.sp.scaled(),
		letterSpacing = 0.1.sp.scaled(),
	),
	bodyLarge = TextStyle(
		fontFamily = sans,
		fontWeight = FontWeight.Normal,
		fontSize = 16.sp.scaled(),
		lineHeight = 24.sp.scaled(),
		letterSpacing = 0.16.sp.scaled(),
	),
	bodyMedium = TextStyle(
		fontFamily = sans,
		fontWeight = FontWeight.Normal,
		fontSize = 14.sp.scaled(),
		lineHeight = 20.sp.scaled(),
		letterSpacing = 0.25.sp.scaled(),
	),
	bodySmall = TextStyle(
		fontFamily = sans,
		fontWeight = FontWeight.Normal,
		fontSize = 12.sp.scaled(),
		lineHeight = 16.sp.scaled(),
		letterSpacing = 0.4.sp.scaled(),
	),
	labelLarge = TextStyle(
		fontFamily = sans,
		fontWeight = FontWeight.Bold,
		fontSize = 14.sp.scaled(),
		lineHeight = 20.sp.scaled(),
		letterSpacing = 0.7.sp.scaled(),
	),
	labelMedium = TextStyle(
		fontFamily = sans,
		fontWeight = FontWeight.Bold,
		fontSize = 12.sp.scaled(),
		lineHeight = 16.sp.scaled(),
		letterSpacing = 0.7.sp.scaled(),
	),
	labelSmall = TextStyle(
		fontFamily = sans,
		fontWeight = FontWeight.Bold,
		fontSize = 11.sp.scaled(),
		lineHeight = 16.sp.scaled(),
		letterSpacing = 0.7.sp.scaled(),
	),
)

// ── Custom styles beyond the M3 scale ────────────────────────────────────────
// Use these for Nym-specific components that don't map to a standard M3 role.

object CustomTypography {
	val labelHuge = TextStyle(
		fontFamily = sans,
		fontWeight = FontWeight.Bold,
		fontSize = 18.sp.scaled(),
		lineHeight = 24.sp.scaled(),
		textAlign = TextAlign.Center,
	)

	val titleMediumPlus = TextStyle(
		fontFamily = sans,
		fontWeight = FontWeight.Medium,
		fontSize = 20.sp.scaled(),
		lineHeight = 24.sp.scaled(),
	)

	val buttonMain = TextStyle(
		fontFamily = sans,
		fontWeight = FontWeight.Medium,
		fontSize = 16.sp.scaled(),
		lineHeight = 22.4.sp.scaled(),
		letterSpacing = 0.sp,
	)

	// For account IDs, device IDs, IP addresses.
	val mono = TextStyle(
		fontFamily = monoFont,
		fontWeight = FontWeight.Normal,
		fontSize = 13.sp.scaled(),
		lineHeight = 18.sp.scaled(),
		letterSpacing = 0.sp,
	)
}
