package net.nymtech.nymvpn.ui.theme

import android.app.Activity
import android.os.Build
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.material3.dynamicDarkColorScheme
import androidx.compose.material3.dynamicLightColorScheme
import androidx.compose.material3.lightColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.runtime.SideEffect
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.toArgb
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalView
import androidx.core.view.WindowCompat

enum class Theme {
    AUTOMATIC,
    DARK_MODE,
    LIGHT_MODE,
    DYNAMIC,
    ;

    companion object {
        fun default(): Theme = AUTOMATIC
    }
}

// ── Color schemes ─────────────────────────────────────────────────────────────

private val DarkColorScheme = darkColorScheme(
    primary = DarkSchemeBaseColors.primary,
    onPrimary = DarkSchemeBaseColors.onPrimary,
    primaryContainer = DarkSchemeBaseColors.primaryContainer,
    onPrimaryContainer = DarkSchemeBaseColors.onPrimaryContainer,
    secondary = DarkSchemeBaseColors.secondary,
    onSecondary = DarkSchemeBaseColors.onSecondary,
    secondaryContainer = DarkSchemeBaseColors.secondaryContainer,
    onSecondaryContainer = DarkSchemeBaseColors.onSecondaryContainer,
    tertiary = DarkSchemeBaseColors.tertiary,
    onTertiary = DarkSchemeBaseColors.onTertiary,
    tertiaryContainer = DarkSchemeBaseColors.tertiaryContainer,
    onTertiaryContainer = DarkSchemeBaseColors.onTertiaryContainer,
    background = DarkSchemeBaseColors.background,
    onBackground = DarkSchemeBaseColors.onBackground,
    surface = DarkSchemeBaseColors.surface,
    onSurface = DarkSchemeBaseColors.onSurface,
    surfaceVariant = DarkSchemeBaseColors.surfaceVariant,
    onSurfaceVariant = DarkSchemeBaseColors.onSurfaceVariant,
    surfaceTint = DarkSchemeBaseColors.surfaceTint,
    inverseSurface = DarkSchemeBaseColors.inverseSurface,
    inverseOnSurface = DarkSchemeBaseColors.inverseOnSurface,
    inversePrimary = DarkSchemeBaseColors.inversePrimary,
    outline = DarkSchemeBaseColors.outline,
    outlineVariant = DarkSchemeBaseColors.outlineVariant,
    scrim = DarkSchemeBaseColors.scrim,
    surfaceBright = DarkSchemeBaseColors.surfaceBright,
    surfaceDim = DarkSchemeBaseColors.surfaceDim,
    surfaceContainer = DarkSchemeBaseColors.surfaceContainer,
    surfaceContainerHigh = DarkSchemeBaseColors.surfaceContainerHigh,
    surfaceContainerHighest = DarkSchemeBaseColors.surfaceContainerHighest,
    surfaceContainerLow = DarkSchemeBaseColors.surfaceContainerLow,
    surfaceContainerLowest = DarkSchemeBaseColors.surfaceContainerLowest,
    error = DarkSchemeBaseColors.error,
    onError = DarkSchemeBaseColors.onError,
    errorContainer = DarkSchemeBaseColors.errorContainer,
    onErrorContainer = DarkSchemeBaseColors.onErrorContainer,
)

private val LightColorScheme = lightColorScheme(
    primary = LightSchemeBaseColors.primary,
    onPrimary = LightSchemeBaseColors.onPrimary,
    primaryContainer = LightSchemeBaseColors.primaryContainer,
    onPrimaryContainer = LightSchemeBaseColors.onPrimaryContainer,
    secondary = LightSchemeBaseColors.secondary,
    onSecondary = LightSchemeBaseColors.onSecondary,
    secondaryContainer = LightSchemeBaseColors.secondaryContainer,
    onSecondaryContainer = LightSchemeBaseColors.onSecondaryContainer,
    tertiary = LightSchemeBaseColors.tertiary,
    onTertiary = LightSchemeBaseColors.onTertiary,
    tertiaryContainer = LightSchemeBaseColors.tertiaryContainer,
    onTertiaryContainer = LightSchemeBaseColors.onTertiaryContainer,
    background = LightSchemeBaseColors.background,
    onBackground = LightSchemeBaseColors.onBackground,
    surface = LightSchemeBaseColors.surface,
    onSurface = LightSchemeBaseColors.onSurface,
    surfaceVariant = LightSchemeBaseColors.surfaceVariant,
    onSurfaceVariant = LightSchemeBaseColors.onSurfaceVariant,
    surfaceTint = LightSchemeBaseColors.surfaceTint,
    inverseSurface = LightSchemeBaseColors.inverseSurface,
    inverseOnSurface = LightSchemeBaseColors.inverseOnSurface,
    inversePrimary = LightSchemeBaseColors.inversePrimary,
    outline = LightSchemeBaseColors.outline,
    outlineVariant = LightSchemeBaseColors.outlineVariant,
    scrim = LightSchemeBaseColors.scrim,
    surfaceBright = LightSchemeBaseColors.surfaceBright,
    surfaceDim = LightSchemeBaseColors.surfaceDim,
    surfaceContainer = LightSchemeBaseColors.surfaceContainer,
    surfaceContainerHigh = LightSchemeBaseColors.surfaceContainerHigh,
    surfaceContainerHighest = LightSchemeBaseColors.surfaceContainerHighest,
    surfaceContainerLow = LightSchemeBaseColors.surfaceContainerLow,
    surfaceContainerLowest = LightSchemeBaseColors.surfaceContainerLowest,
    error = LightSchemeBaseColors.error,
    onError = LightSchemeBaseColors.onError,
    errorContainer = LightSchemeBaseColors.errorContainer,
    onErrorContainer = LightSchemeBaseColors.onErrorContainer,
)

// ── Theme composable ──────────────────────────────────────────────────────────

@Composable
fun NymVPNTheme(theme: Theme, content: @Composable () -> Unit) {
    val context = LocalContext.current
    var isDark = isSystemInDarkTheme()

    val colorScheme = when (theme) {
        Theme.DARK_MODE -> DarkColorScheme.also { isDark = true }
        Theme.LIGHT_MODE -> LightColorScheme.also { isDark = false }
        Theme.DYNAMIC -> {
            if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S) {
                if (isDark) dynamicDarkColorScheme(context) else dynamicLightColorScheme(context)
            } else {
                if (isDark) DarkColorScheme else LightColorScheme
            }
        }
        Theme.AUTOMATIC -> if (isDark) DarkColorScheme else LightColorScheme
    }

    val nymColors = if (isDark) DarkNymColors else LightNymColors

    val view = LocalView.current
    if (!view.isInEditMode) {
        SideEffect {
            val window = (view.context as Activity).window
            WindowCompat.setDecorFitsSystemWindows(window, false)
            window.navigationBarColor = Color.Transparent.toArgb()
            window.statusBarColor = Color.Transparent.toArgb()
            WindowCompat.getInsetsController(window, window.decorView).apply {
                isAppearanceLightStatusBars = !isDark
                isAppearanceLightNavigationBars = !isDark
            }
        }
    }

    CompositionLocalProvider(LocalNymColors provides nymColors) {
        MaterialTheme(
            colorScheme = colorScheme,
            typography = Typography,
            content = content,
        )
    }
}
