package net.nymtech.qrscanner.ui.theme

import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.darkColorScheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Color

// Always dark since it sits over the camera feed; typography comes from the host theme.
private val ScannerColorScheme = darkColorScheme(
	primary = Color(0xFF5BF0A0),
	onPrimary = Color(0xFF0A0A0A),
	background = Color.Black,
	onBackground = Color.White,
	surface = Color(0xFF1D1D1F),
	onSurface = Color.White,
	onSurfaceVariant = Color(0xFFAEACB1),
	error = Color(0xFFFF4444),
	errorContainer = Color(0xFFCD2C3C),
	onErrorContainer = Color.White,
)

internal object QrScannerColors {
	val scrim = Color(0x99000000)
	val overlayButton = Color(0x66000000)
	val onOverlay = Color.White
}

@Composable
internal fun QrScannerTheme(content: @Composable () -> Unit) {
	MaterialTheme(
		colorScheme = ScannerColorScheme,
		typography = MaterialTheme.typography,
		content = content,
	)
}
