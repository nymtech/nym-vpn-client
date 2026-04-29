package net.nymtech.nymvpn.ui.theme

import androidx.compose.runtime.Immutable
import androidx.compose.runtime.staticCompositionLocalOf
import androidx.compose.ui.graphics.Color


val Green = Color(0xFF5BF0A0)       // primary / button fill — both modes
val GreenDark = Color(0xFF1A9B61)   // primary text/icon in light mode
val GreenOnPrimary = Color(0xFF052E18)

internal object DarkSchemeBaseColors {
    val primary = Green
    val onPrimary = GreenOnPrimary
    // greenIconBackground
    val primaryContainer = Color(0xFF0A4A28)
    val onPrimaryContainer = Color(0xFFB8FDD5)

    val secondary = Color(0xFF48484A)
    val onSecondary = Color(0xFFAEAEB2)
    val secondaryContainer = Color(0xFF3A3A3C)
    val onSecondaryContainer = Color(0xFFCCCCCF)

    val tertiary = Green
    val onTertiary = GreenOnPrimary
    val tertiaryContainer = Color(0xFF0A3D22)
    val onTertiaryContainer = Color(0xFFB8FDD5)

    // errorPillText
    val background = Color(0xFF0D0D0F)
    val onBackground = Color(0xFFFFFFFF)

    val surface = Color(0xFF1C1C1E)
    val onSurface = Color(0xFFE5E5E7)
    // statusDefaultBg
    val surfaceVariant = Color(0xFF2C2C2E)
    // stateLabel
    val onSurfaceVariant = Color(0xFF8B8B90)
    val surfaceTint = Green
    val surfaceBright = Color(0xFF3A3A3C)
    val surfaceDim = Color(0xFF0D0D0F)
    val surfaceContainer = Color(0xFF2C2C2E)
    val surfaceContainerHigh = Color(0xFF363638)
    val surfaceContainerHighest = Color(0xFF424244)
    val surfaceContainerLow = Color(0xFF1C1C1E)
    val surfaceContainerLowest = Color(0xFF0D0D0F)

    val outline = Color(0xFF3C3C3E)
    val outlineVariant = Color(0xFF2C2C2E)
    val scrim = Color(0xFF000000)

    val inverseSurface = Color(0xFFE5E5E7)
    val inverseOnSurface = Color(0xFF1C1C1E)
    val inversePrimary = GreenDark

    // redIcon, CustomColors.error, CustomColors.disconnect
    val error = Color(0xFFE73E14)
    val onError = Color(0xFFFFFFFF)
    // redIconBackground, statusErrorBg
    val errorContainer = Color(0xFF5C1208)
    val onErrorContainer = Color(0xFFFFBDAB)
}

internal object LightSchemeBaseColors {
    val primary = Green
    val onPrimary = GreenOnPrimary
    // greenIconBackground
    val primaryContainer = Color(0xFFC2FFE0)
    val onPrimaryContainer = Color(0xFF002D14)

    val secondary = Color(0xFF8E8E93)
    val onSecondary = Color(0xFFFFFFFF)
    val secondaryContainer = Color(0xFFE5E5EA)
    val onSecondaryContainer = Color(0xFF3A3A3C)

    val tertiary = GreenDark
    val onTertiary = Color(0xFFFFFFFF)
    val tertiaryContainer = Color(0xFFC2FFE0)
    val onTertiaryContainer = Color(0xFF002D14)

    // errorPillText
    val background = Color(0xFFF2F2F7)
    val onBackground = Color(0xFF0D0D0F)

    val surface = Color(0xFFFFFFFF)
    val onSurface = Color(0xFF0D0D0F)
    // statusDefaultBg
    val surfaceVariant = Color(0xFFEBEBF0)
    // stateLabel
    val onSurfaceVariant = Color(0xFF6B6B70)
    val surfaceTint = GreenDark
    val surfaceBright = Color(0xFFFFFFFF)
    val surfaceDim = Color(0xFFDFE0E5)
    val surfaceContainer = Color(0xFFEBEBF0)
    val surfaceContainerHigh = Color(0xFFE5E5EA)
    val surfaceContainerHighest = Color(0xFFE0E0E5)
    val surfaceContainerLow = Color(0xFFF0F0F5)
    val surfaceContainerLowest = Color(0xFFFFFFFF)

    val outline = Color(0xFFB4B4B9)
    val outlineVariant = Color(0xFFDFDFE4)
    val scrim = Color(0xFF000000)

    val inverseSurface = Color(0xFF1C1C1E)
    val inverseOnSurface = Color(0xFFF2F2F7)
    val inversePrimary = Green

    // redIcon, CustomColors.error, CustomColors.disconnect
    val error = Color(0xFFE73E14)
    val onError = Color(0xFFFFFFFF)
    // redIconBackground, statusErrorBg
    val errorContainer = Color(0xFFFFDAD0)
    val onErrorContainer = Color(0xFF5C1208)
}

object CustomColors {
    val warning = Color(0xFFFFB400)
    val warningAmber = Color(0xFFFB6E4E)
    val pulse = Color(0xFF7075FF)
    val buttonRedTransparent = Color(0x1AED5060)
    val buttonRedTransparentBorder = Color(0xFFED5060)
}

@Immutable
data class NymColors(
    val snackBarBackground: Color = Color.Unspecified,
    val snackbarText: Color = Color.Unspecified,
    val iconBorder: Color = Color.Unspecified,
    val iconBackground: Color = Color.Unspecified,
    val greyIconBackground: Color = Color.Unspecified,
    val greyIcon: Color = Color.Unspecified,
    val greenIcon: Color = Color.Unspecified,
    val borderCyan: Color = Color.Unspecified,
    val labelCyan: Color = Color.Unspecified,
    val statusConnectedBg: Color = Color.Unspecified,
    val fastFill: Color = Color.Unspecified,
    val anonFill: Color = Color.Unspecified,   // semi-transparent onSurfaceVariant
    val errorFill: Color = Color.Unspecified,  // semi-transparent error
    val trackIdle: Color = Color.Unspecified,
)

val DarkNymColors = NymColors(
    snackBarBackground = Color(0xFF2C2C2E),
    snackbarText       = Color(0xFFE5E5E7),
    iconBorder         = Color(0x405BF0A0),
    iconBackground     = Color(0x265BF0A0),
    greyIconBackground = Color(0xFF2C2C2E),
    greyIcon           = Color(0xFF8B8B90),
    greenIcon          = Color(0xFF5BF0A0),
    borderCyan         = Color(0x8098DDFF),
    labelCyan          = Color(0xFF98DDFF),
    statusConnectedBg  = Color(0x1A5BF0A0),
    fastFill           = Color(0xFF5BF0A0),
    anonFill           = Color(0x998B8B90),
    errorFill          = Color(0x99E73E14),
    trackIdle          = Color(0x26FFFFFF),
)

val LightNymColors = NymColors(
    snackBarBackground = Color(0xFF1C1C1E),
    snackbarText       = Color(0xFFF2F2F7),
    iconBorder         = Color(0x401A9B61),
    iconBackground     = Color(0x261A9B61),
    greyIconBackground = Color(0xFFE5E5EA),
    greyIcon           = Color(0xFF8E8E93),
    greenIcon          = Color(0xFF1A9B61),
    borderCyan         = Color(0x8000A3F5),
    labelCyan          = Color(0xFF00A3F5),
    statusConnectedBg  = Color(0x1A1A9B61),
    fastFill           = Color(0xFF1A9B61),
    anonFill           = Color(0x996B6B70),
    errorFill          = Color(0x99E73E14),
    trackIdle          = Color(0x1F0A0A0A),
)

val LocalNymColors = staticCompositionLocalOf { NymColors() }
