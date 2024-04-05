package net.nymtech.nymvpn.ui.theme

import androidx.compose.material3.Typography
import androidx.compose.ui.text.TextStyle
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.sp
import net.nymtech.nymvpn.util.scaled

// Set of Material typography styles to start with
val Typography =
    Typography(
        bodyLarge =
        TextStyle(
            fontFamily = FontFamily.Default,
            fontWeight = FontWeight.Normal,
            fontSize = 16.sp.scaled(),
            lineHeight = 24.sp.scaled(),
            letterSpacing = 0.5.sp.scaled()
        ),
        bodySmall =
        TextStyle(
            fontSize = 12.sp.scaled(),
            lineHeight = 16.sp.scaled(),
            fontWeight = FontWeight(400),
            letterSpacing = 0.4.sp.scaled(),
        ),
        titleLarge =
        TextStyle(
            fontSize = 22.sp.scaled(),
            lineHeight = 28.sp.scaled(),
            fontWeight = FontWeight(400),
        ),
        titleMedium =
        TextStyle(
            fontSize = 16.sp.scaled(),
            lineHeight = 24.sp.scaled(),
            fontWeight = FontWeight(600),
            letterSpacing = 0.15.sp.scaled(),
        ),
        bodyMedium =
        TextStyle(
            fontSize = 14.sp.scaled(),
            lineHeight = 20.sp.scaled(),
            fontWeight = FontWeight(400),
            letterSpacing = 0.25.sp.scaled(),
        ),
        labelSmall =
        TextStyle(
            fontSize = 11.sp.scaled(),
            lineHeight = 16.sp.scaled(),
            fontWeight = FontWeight(500),
            letterSpacing = 0.5.sp.scaled(),
        ),
        headlineSmall = TextStyle(
            fontSize = 24.sp.scaled(),
            lineHeight = 32.sp.scaled(),
            fontWeight = FontWeight(400),
            textAlign = TextAlign.Center,
        ),
        labelLarge =
        TextStyle(
            fontSize = 14.sp.scaled(),
            lineHeight = 20.sp.scaled(),
            fontWeight = FontWeight(700),
            textAlign = TextAlign.Center,
            letterSpacing = 0.1.sp.scaled(),
        )
    )
