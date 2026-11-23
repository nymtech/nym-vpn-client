package net.nymtech.qrscanner.ui.components

import androidx.camera.view.PreviewView
import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.blur
import androidx.compose.ui.graphics.FilterQuality
import androidx.compose.ui.graphics.ImageBitmap
import androidx.compose.ui.graphics.asImageBitmap
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.unit.dp
import androidx.compose.ui.viewinterop.AndroidView
import androidx.lifecycle.Observer
import androidx.lifecycle.compose.LocalLifecycleOwner
import net.nymtech.qrscanner.camera.QrCameraSession

private val StillBlurRadius = 16.dp

@Composable
internal fun QrCameraPreview(
	scanning: Boolean,
	paused: Boolean,
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
	val currentPaused by rememberUpdatedState(paused)

	val previewView = remember(context) {
		PreviewView(context).apply {
			// TextureView keeps the preview inside the window, so FLAG_SECURE covers it.
			implementationMode = PreviewView.ImplementationMode.COMPATIBLE
			scaleType = PreviewView.ScaleType.FILL_CENTER
		}
	}
	var session by remember { mutableStateOf<QrCameraSession?>(null) }
	var still by remember { mutableStateOf<ImageBitmap?>(null) }

	DisposableEffect(lifecycleOwner, previewView) {
		val listener = object : QrCameraSession.Listener {
			override fun onCodeScan(text: String) = currentOnCodeScan(text)

			override fun onTorchAvailabilityChange(available: Boolean) = currentOnTorchAvailabilityChange(available)

			override fun onCameraError() = currentOnCameraError()
		}
		val newSession = QrCameraSession(context, lifecycleOwner, previewView, listener)
		newSession.start()
		session = newSession
		// The still stays up until live frames are back.
		val streamObserver = Observer<PreviewView.StreamState> { state ->
			if (state == PreviewView.StreamState.STREAMING && !currentPaused) still = null
		}
		previewView.previewStreamState.observe(lifecycleOwner, streamObserver)
		onDispose {
			previewView.previewStreamState.removeObserver(streamObserver)
			newSession.release()
			session = null
			still = null
		}
	}

	LaunchedEffect(session, torchEnabled) {
		session?.setTorchEnabled(torchEnabled)
	}

	LaunchedEffect(session, paused, scanning) {
		val activeSession = session ?: return@LaunchedEffect
		if (paused) {
			activeSession.pause()?.let { still = it.asImageBitmap() }
		} else {
			activeSession.resume()
			if (!scanning) activeSession.stopScanning()
		}
	}

	Box(modifier = modifier) {
		AndroidView(factory = { previewView }, modifier = Modifier.fillMaxSize())
		still?.let { bitmap ->
			// Upscaling blurs on every API level; Modifier.blur adds to it from API 31.
			Image(
				bitmap = bitmap,
				contentDescription = null,
				contentScale = ContentScale.FillBounds,
				filterQuality = FilterQuality.Low,
				modifier = Modifier
					.fillMaxSize()
					.blur(StillBlurRadius),
			)
		}
	}
}
