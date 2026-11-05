package net.nymtech.qrscanner

import android.content.Context
import android.content.pm.PackageManager
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import net.nymtech.qrscanner.model.QrScanMode
import net.nymtech.qrscanner.model.QrScanResult
import net.nymtech.qrscanner.ui.QrScannerScreen
import net.nymtech.qrscanner.ui.theme.QrScannerTheme

// Full-screen scanner. The host must block screenshots and overlays while it is shown.
@Composable
fun QrScanner(mode: QrScanMode, validator: QrScanValidator, onResult: (QrScanResult) -> Unit, onClose: () -> Unit, modifier: Modifier = Modifier) {
	QrScannerTheme {
		QrScannerScreen(
			mode = mode,
			validator = validator,
			onResult = onResult,
			onClose = onClose,
			modifier = modifier,
		)
	}
}

fun isQrCameraAvailable(context: Context): Boolean = context.packageManager.hasSystemFeature(PackageManager.FEATURE_CAMERA_ANY)
