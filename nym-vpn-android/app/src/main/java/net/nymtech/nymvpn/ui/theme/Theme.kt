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
import androidx.compose.runtime.SideEffect
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.toArgb
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalView
import androidx.core.view.WindowCompat

enum class Theme {
    DARK_MODE,
    LIGHT_MODE,
    AUTOMATIC
}

private val DarkColorScheme = darkColorScheme(
    background = ThemeColors.Dark.background,
    surface = ThemeColors.Dark.surface,
    primary = ThemeColors.Dark.primary,
    secondary = ThemeColors.Dark.secondary,
    onBackground = ThemeColors.Dark.onBackground,
    onSurface = ThemeColors.Dark.onSurface,
    onPrimary = ThemeColors.Dark.onPrimary,
    onSurfaceVariant = ThemeColors.Dark.onSurfaceVariant,
    onSecondary = ThemeColors.Dark.onSecondary
)

private val LightColorScheme = lightColorScheme(
    background = ThemeColors.Light.background,
    surface = ThemeColors.Light.surface,
    primary = ThemeColors.Light.primary,
    secondary = ThemeColors.Light.secondary,
    onBackground = ThemeColors.Light.onBackground,
    onSurface = ThemeColors.Light.onSurface,
    onPrimary = ThemeColors.Light.onPrimary,
    onSurfaceVariant = ThemeColors.Light.onSurfaceVariant,
    onSecondary = ThemeColors.Light.onSecondary
)

@Composable
fun NymVPNTheme(
    theme: Theme,
    // Dynamic color is available on Android 12+
    // disable for now..
    dynamicColor: Boolean = false,
    content: @Composable () -> Unit
) {
    val context = LocalContext.current

    val darkTheme = when(theme) {
        Theme.AUTOMATIC -> isSystemInDarkTheme()
        Theme.DARK_MODE -> true
        Theme.LIGHT_MODE -> false
    }

    val colorScheme = when {
        dynamicColor && Build.VERSION.SDK_INT >= Build.VERSION_CODES.S -> {
            if (darkTheme) dynamicDarkColorScheme(context) else dynamicLightColorScheme(context)
        }
        darkTheme -> DarkColorScheme
        else -> LightColorScheme
    }

    val view = LocalView.current
    if (!view.isInEditMode) {
        SideEffect {
            val window = (view.context as Activity).window
            WindowCompat.setDecorFitsSystemWindows(window, false)
            window.statusBarColor = Color.Transparent.toArgb()
            window.navigationBarColor = Color.Transparent.toArgb()
            WindowCompat.getInsetsController(window, window.decorView).isAppearanceLightStatusBars =
                !darkTheme
        }
    }


    MaterialTheme(
        colorScheme = colorScheme,
        typography = Typography,
        content = content
    )
}