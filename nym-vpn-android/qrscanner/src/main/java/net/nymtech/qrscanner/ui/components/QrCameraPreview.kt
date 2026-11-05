package net.nymtech.qrscanner.ui.components

import androidx.camera.view.PreviewView
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.viewinterop.AndroidView
import androidx.lifecycle.compose.LocalLifecycleOwner
import net.nymtech.qrscanner.camera.QrCameraSession

@Composable
internal fun QrCameraPreview(
	scanning: Boolean,
	torchEnabled: Boolean,
	onCodeScan: (String) -> Unit,
	onTorchAvailabilityChange: (Boolean) -> Unit,
	onCameraError: () -> Unit,
	modifier: Modifier = Modifier,
) {
	val context = LocalContext.current
	val lifecycleOwner = LocalLifecycleOwner.current
	val currentOnCodeScan by rememberUpdatedState(onCodeScan)
	val currentOnTorchAvailabilityChange by rememberUpdatedState(onTorchAvailabilityChange)
	val currentOnCameraError by rememberUpdatedState(onCameraError)

	val previewView = remember(context) {
		PreviewView(context).apply {
			// TextureView keeps the preview inside the window, so FLAG_SECURE covers it.
			implementationMode = PreviewView.ImplementationMode.COMPATIBLE
			scaleType = PreviewView.ScaleType.FILL_CENTER
		}
	}
	var session by remember { mutableStateOf<QrCameraSession?>(null) }

	DisposableEffect(lifecycleOwner, previewView) {
		val listener = object : QrCameraSession.Listener {
			override fun onCodeScan(text: String) = currentOnCodeScan(text)

			override fun onTorchAvailabilityChange(available: Boolean) = currentOnTorchAvailabilityChange(available)

			override fun onCameraError() = currentOnCameraError()
		}
		val newSession = QrCameraSession(context, lifecycleOwner, previewView, listener)
		newSession.start()
		session = newSession
		onDispose {
			newSession.release()
			session = null
		}
	}

	LaunchedEffect(session, torchEnabled) {
		session?.setTorchEnabled(torchEnabled)
	}

	LaunchedEffect(session, scanning) {
		if (!scanning) session?.stopScanning()
	}

	AndroidView(factory = { previewView }, modifier = modifier)
}
