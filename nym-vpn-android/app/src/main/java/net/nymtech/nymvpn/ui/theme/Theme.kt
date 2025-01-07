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
	AUTOMATIC,
	DARK_MODE,
	DYNAMIC,
	LIGHT_MODE,
	;

	companion object {
		fun default(): Theme {
			return AUTOMATIC
		}
	}
}

private val DarkColorScheme =
	darkColorScheme(
		background = ThemeColors.Dark.background,
		surface = ThemeColors.Dark.surface,
		primary = ThemeColors.Dark.primary,
		secondary = ThemeColors.Dark.secondary,
		onBackground = ThemeColors.Dark.onBackground,
		onSurface = ThemeColors.Dark.onSurface,
		onPrimary = ThemeColors.Dark.onPrimary,
		onSurfaceVariant = ThemeColors.Dark.onSurfaceVariant,
		onSecondary = ThemeColors.Dark.onSecondary,
		surfaceContainer = ThemeColors.Dark.surfaceContainer,
		tertiary = ThemeColors.Dark.tertiary,
	)

private val LightColorScheme =
	lightColorScheme(
		background = ThemeColors.Light.background,
		surface = ThemeColors.Light.surface,
		primary = ThemeColors.Light.primary,
		secondary = ThemeColors.Light.secondary,
		onBackground = ThemeColors.Light.onBackground,
		onSurface = ThemeColors.Light.onSurface,
		onPrimary = ThemeColors.Light.onPrimary,
		onSurfaceVariant = ThemeColors.Light.onSurfaceVariant,
		onSecondary = ThemeColors.Light.onSecondary,
		surfaceContainer = ThemeColors.Light.surfaceContainer,
		tertiary = ThemeColors.Light.tertiary,
	)

@Composable
fun NymVPNTheme(theme: Theme, content: @Composable () -> Unit) {
	val context = LocalContext.current
	var isDark = isSystemInDarkTheme()

	val colorScheme =
		when (theme) {
			Theme.DARK_MODE -> DarkColorScheme.also { isDark = true }
			Theme.LIGHT_MODE -> LightColorScheme.also { isDark = false }
			Theme.DYNAMIC, Theme.AUTOMATIC -> {
				if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.S && theme == Theme.DYNAMIC) {
					if (isDark) dynamicDarkColorScheme(context) else dynamicLightColorScheme(context)
				} else {
					if (isDark) DarkColorScheme else LightColorScheme
				}
			}
		}
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

	MaterialTheme(
		colorScheme = colorScheme,
		typography = Typography,
		content = content,
	)
}
