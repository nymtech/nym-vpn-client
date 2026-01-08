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

// Set of Material typography styles to start with
val Typography =
	Typography(
		bodyLarge =
		TextStyle(
			fontWeight = FontWeight.Normal,
			fontSize = 16.sp.scaled(),
			lineHeight = 24.sp.scaled(),
			letterSpacing = 0.5.sp.scaled(),
			fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
		),
		bodySmall =
		TextStyle(
			fontSize = 12.sp.scaled(),
			lineHeight = 16.sp.scaled(),
			fontWeight = FontWeight(400),
			letterSpacing = 0.4.sp.scaled(),
			fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
		),
		titleLarge =
		TextStyle(
			fontSize = 24.sp.scaled(),
			lineHeight = 28.8.sp.scaled(),
			fontWeight = FontWeight(500),
		),
		titleMedium =
		TextStyle(
			fontSize = 16.sp.scaled(),
			lineHeight = 24.sp.scaled(),
			fontWeight = FontWeight(500),
			letterSpacing = 0.15.sp.scaled(),
		),
		bodyMedium =
		TextStyle(
			fontSize = 14.sp.scaled(),
			lineHeight = 20.sp.scaled(),
			fontWeight = FontWeight(400),
			fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
			letterSpacing = 0.25.sp.scaled(),
		),
		labelSmall =
		TextStyle(
			fontSize = 11.sp.scaled(),
			lineHeight = 16.sp.scaled(),
			fontWeight = FontWeight(500),
			letterSpacing = 0.5.sp.scaled(),
		),
		headlineSmall =
		TextStyle(
			fontSize = 24.sp.scaled(),
			lineHeight = 32.sp.scaled(),
			fontWeight = FontWeight(400),
			textAlign = TextAlign.Center,
		),
		labelMedium = TextStyle(
			fontSize = 12.sp.scaled(),
			lineHeight = 16.sp.scaled(),
			fontWeight = FontWeight(500),
			textAlign = TextAlign.Center,
			letterSpacing = 0.5.sp,
		),
		labelLarge = TextStyle(
			fontSize = 14.sp.scaled(),
			lineHeight = 20.sp.scaled(),
			fontWeight = FontWeight(700),
			textAlign = TextAlign.Center,
			letterSpacing = 0.1.sp,
		),
	)

object CustomTypography {
	val labelHuge = TextStyle(
		fontSize = 18.sp.scaled(),
		lineHeight = 24.sp.scaled(),
		fontWeight = FontWeight(700),
		textAlign = TextAlign.Center,
	)
	val titleMediumPlus = TextStyle(
		fontSize = 20.sp.scaled(),
		lineHeight = 24.sp.scaled(),
		fontWeight = FontWeight(500),
	)

	val buttonMain = TextStyle(
		fontSize = 16.sp.scaled(),
		lineHeight = 22.4.sp.scaled(),
		fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
		fontWeight = FontWeight(500),
	)
}
