package net.nymtech.nymvpn.ui.theme

import android.app.Activity
import android.content.Context
import android.content.ContextWrapper
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

private val DarkColorScheme = darkColorScheme(
	primary = DarkSchemeBaseColors.primary,
	onPrimary = DarkSchemeBaseColors.onPrimary,
	primaryContainer = DarkSchemeBaseColors.primaryContainer,
	onPrimaryContainer = DarkSchemeBaseColors.onPrimaryContainer,
	secondary = DarkSchemeBaseColors.secondary,
	onSecondary = DarkSchemeBaseColors.onSecondary,
	tertiary = DarkSchemeBaseColors.tertiary,
	background = DarkSchemeBaseColors.background,
	onBackground = DarkSchemeBaseColors.onBackground,
	surface = DarkSchemeBaseColors.surface,
	onSurface = DarkSchemeBaseColors.onSurface,
	inverseSurface = DarkSchemeBaseColors.inverseSurface,
	error = DarkSchemeBaseColors.error,
	errorContainer = DarkSchemeBaseColors.errorContainer,
	onErrorContainer = DarkSchemeBaseColors.onErrorContainer,
	outline = DarkSchemeBaseColors.outline,
)

private val LightColorScheme = lightColorScheme(
	primary = LightSchemeBaseColors.primary,
	onPrimary = LightSchemeBaseColors.onPrimary,
	primaryContainer = LightSchemeBaseColors.primaryContainer,
	onPrimaryContainer = LightSchemeBaseColors.onPrimaryContainer,
	secondary = LightSchemeBaseColors.secondary,
	onSecondary = LightSchemeBaseColors.onSecondary,
	tertiary = LightSchemeBaseColors.tertiary,
	background = LightSchemeBaseColors.background,
	onBackground = LightSchemeBaseColors.onBackground,
	surface = LightSchemeBaseColors.surface,
	onSurface = LightSchemeBaseColors.onSurface,
	inverseSurface = LightSchemeBaseColors.inverseSurface,
	error = LightSchemeBaseColors.error,
	errorContainer = LightSchemeBaseColors.errorContainer,
	onErrorContainer = LightSchemeBaseColors.onErrorContainer,
	outline = LightSchemeBaseColors.outline,
)

@Composable
fun NymVPNTheme(theme: Theme, content: @Composable () -> Unit) {
	val context = LocalContext.current
	val systemDark = isSystemInDarkTheme()
	val isDark = when (theme) {
		Theme.DARK_MODE -> true
		Theme.LIGHT_MODE -> false
		else -> systemDark
	}

	val colorScheme = when (theme) {
		Theme.DARK_MODE -> DarkColorScheme
		Theme.LIGHT_MODE -> LightColorScheme
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

	tailrec fun Context.findActivity(): Activity? = when (this) {
		is Activity -> this
		is ContextWrapper -> baseContext.findActivity()
		else -> null
	}

	val view = LocalView.current
	if (!view.isInEditMode) {
		SideEffect {
			val window = view.context.findActivity()?.window ?: return@SideEffect
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
