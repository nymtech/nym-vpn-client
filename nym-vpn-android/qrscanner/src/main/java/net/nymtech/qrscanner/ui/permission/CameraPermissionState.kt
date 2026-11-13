package net.nymtech.qrscanner.ui.permission

import android.Manifest
import android.content.Context
import android.content.pm.PackageManager
import androidx.activity.compose.LocalActivity
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.runtime.Composable
import androidx.compose.runtime.Immutable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.platform.LocalContext
import androidx.core.content.ContextCompat
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.compose.LifecycleEventEffect

internal enum class CameraPermissionStatus {
	GRANTED,
	NOT_GRANTED,
	DENIED,
	PERMANENTLY_DENIED,
}

@Immutable
internal class CameraPermissionState(val status: CameraPermissionStatus, val request: () -> Unit)

@Composable
internal fun rememberCameraPermissionState(): CameraPermissionState {
	val context = LocalContext.current
	val activity = LocalActivity.current
	var status by rememberSaveable { mutableStateOf(context.cameraPermissionStatus()) }
	var requestedOnce by rememberSaveable { mutableStateOf(false) }

	val launcher = rememberLauncherForActivityResult(ActivityResultContracts.RequestPermission()) { granted ->
		status = when {
			granted -> CameraPermissionStatus.GRANTED
			activity?.shouldShowRequestPermissionRationale(Manifest.permission.CAMERA) == true -> CameraPermissionStatus.DENIED
			else -> CameraPermissionStatus.PERMANENTLY_DENIED
		}
	}

	LaunchedEffect(Unit) {
		if (status != CameraPermissionStatus.GRANTED && !requestedOnce) {
			requestedOnce = true
			launcher.launch(Manifest.permission.CAMERA)
		}
	}

	// Picks up a grant or revoke made in system settings.
	LifecycleEventEffect(Lifecycle.Event.ON_RESUME) {
		val current = context.cameraPermissionStatus()
		if (current == CameraPermissionStatus.GRANTED || status == CameraPermissionStatus.GRANTED) status = current
	}

	return remember(status, launcher) {
		CameraPermissionState(status) { launcher.launch(Manifest.permission.CAMERA) }
	}
}

private fun Context.cameraPermissionStatus(): CameraPermissionStatus = if (ContextCompat.checkSelfPermission(this, Manifest.permission.CAMERA) == PackageManager.PERMISSION_GRANTED) {
	CameraPermissionStatus.GRANTED
} else {
	CameraPermissionStatus.NOT_GRANTED
}
