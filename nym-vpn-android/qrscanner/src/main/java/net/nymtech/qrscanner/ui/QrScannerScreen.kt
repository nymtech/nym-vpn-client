package net.nymtech.qrscanner.ui

import android.content.ActivityNotFoundException
import android.content.Context
import android.content.Intent
import android.net.Uri
import android.provider.Settings
import androidx.activity.compose.BackHandler
import androidx.activity.compose.LocalActivity
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Warning
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.hapticfeedback.HapticFeedbackType
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalHapticFeedback
import androidx.compose.ui.platform.LocalView
import androidx.compose.ui.res.stringResource
import androidx.core.view.WindowCompat
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.viewmodel.compose.viewModel
import net.nymtech.qrscanner.QrScanValidator
import net.nymtech.qrscanner.R
import net.nymtech.qrscanner.model.QrScanAction
import net.nymtech.qrscanner.model.QrScanMode
import net.nymtech.qrscanner.model.QrScanResult
import net.nymtech.qrscanner.ui.components.QrCameraPreview
import net.nymtech.qrscanner.ui.components.QrPermissionContent
import net.nymtech.qrscanner.ui.components.QrScannerMessage
import net.nymtech.qrscanner.ui.components.QrScannerOverlay
import net.nymtech.qrscanner.ui.components.QrScannerTopBar
import net.nymtech.qrscanner.ui.permission.CameraPermissionStatus
import net.nymtech.qrscanner.ui.permission.rememberCameraPermissionState
import timber.log.Timber

@Composable
internal fun QrScannerScreen(
	mode: QrScanMode,
	validator: QrScanValidator,
	onResult: (QrScanResult) -> Unit,
	onClose: () -> Unit,
	secondaryAction: QrScanAction?,
	paused: Boolean,
	modifier: Modifier = Modifier,
	viewModel: QrScannerViewModel = viewModel(key = "qr_scanner_${mode.name}") { QrScannerViewModel(mode, validator) },
) {
	val uiState by viewModel.uiState.collectAsStateWithLifecycle()
	val haptics = LocalHapticFeedback.current

	BackHandler(onBack = onClose)
	LightSystemBarIcons()

	LaunchedEffect(uiState.result) {
		val result = uiState.result ?: return@LaunchedEffect
		haptics.performHapticFeedback(HapticFeedbackType.Confirm)
		onResult(result)
	}

	QrScannerScreen(
		mode = mode,
		uiState = uiState,
		onCodeScan = viewModel::onCodeScan,
		onClose = onClose,
		secondaryAction = secondaryAction,
		paused = paused,
		modifier = modifier,
	)
}

@Composable
internal fun QrScannerScreen(
	mode: QrScanMode,
	uiState: QrScannerUiState,
	onCodeScan: (String) -> Unit,
	onClose: () -> Unit,
	secondaryAction: QrScanAction?,
	paused: Boolean,
	modifier: Modifier = Modifier,
) {
	val context = LocalContext.current
	val permission = rememberCameraPermissionState()
	var cameraError by remember { mutableStateOf(false) }
	var torchAvailable by remember { mutableStateOf(false) }
	var torchEnabled by remember { mutableStateOf(false) }
	val cameraActive = permission.status == CameraPermissionStatus.GRANTED && !cameraError

	LaunchedEffect(paused) {
		if (paused) torchEnabled = false
	}

	Box(
		modifier = modifier
			.fillMaxSize()
			.background(Color.Black),
	) {
		when {
			cameraError -> QrScannerMessage(
				icon = Icons.Outlined.Warning,
				title = stringResource(R.string.qr_scanner_camera_error_title),
				body = stringResource(R.string.qr_scanner_camera_error_body),
				secondaryAction = secondaryAction,
			)
			cameraActive -> {
				QrCameraPreview(
					scanning = uiState.result == null,
					paused = paused,
					torchEnabled = torchEnabled,
					onCodeScan = onCodeScan,
					onTorchAvailabilityChange = { torchAvailable = it },
					onCameraError = { cameraError = true },
					modifier = Modifier.fillMaxSize(),
				)
				if (!paused) {
					QrScannerOverlay(
						hintRes = mode.hintRes,
						invalidCodeRes = uiState.invalidCodeRes,
						showInvalidCode = uiState.showInvalidCode,
						validating = uiState.validating,
						secondaryAction = secondaryAction,
					)
				}
			}
			else -> QrPermissionContent(
				status = permission.status,
				onRequestPermission = permission.request,
				onOpenSettings = { context.openAppSettings() },
				secondaryAction = secondaryAction,
			)
		}
		QrScannerTopBar(
			showTorch = cameraActive && torchAvailable && !paused,
			torchEnabled = torchEnabled,
			onClose = onClose,
			onTorchToggle = { torchEnabled = !torchEnabled },
			modifier = Modifier.align(Alignment.TopCenter),
		)
	}
}

// The host's bar style may be for a light theme; restore it on exit.
@Composable
private fun LightSystemBarIcons() {
	val view = LocalView.current
	val window = LocalActivity.current?.window
	if (view.isInEditMode || window == null) return
	DisposableEffect(window, view) {
		val controller = WindowCompat.getInsetsController(window, view)
		val lightStatusBars = controller.isAppearanceLightStatusBars
		val lightNavigationBars = controller.isAppearanceLightNavigationBars
		controller.isAppearanceLightStatusBars = false
		controller.isAppearanceLightNavigationBars = false
		onDispose {
			controller.isAppearanceLightStatusBars = lightStatusBars
			controller.isAppearanceLightNavigationBars = lightNavigationBars
		}
	}
}

private fun Context.openAppSettings() {
	val intent = Intent(Settings.ACTION_APPLICATION_DETAILS_SETTINGS, Uri.fromParts("package", packageName, null))
	try {
		startActivity(intent)
	} catch (e: ActivityNotFoundException) {
		Timber.w(e, "No app settings screen available")
	}
}
