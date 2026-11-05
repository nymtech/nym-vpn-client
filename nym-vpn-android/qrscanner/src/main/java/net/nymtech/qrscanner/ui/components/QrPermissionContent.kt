package net.nymtech.qrscanner.ui.components

import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.NoPhotography
import androidx.compose.material.icons.outlined.PhotoCamera
import androidx.annotation.StringRes
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import net.nymtech.qrscanner.R
import net.nymtech.qrscanner.ui.permission.CameraPermissionStatus

@Composable
internal fun QrPermissionContent(
	status: CameraPermissionStatus,
	onRequestPermission: () -> Unit,
	onOpenSettings: () -> Unit,
	@StringRes fallbackRes: Int?,
	onFallback: () -> Unit,
	modifier: Modifier = Modifier,
) {
	if (status == CameraPermissionStatus.PERMANENTLY_DENIED) {
		QrScannerMessage(
			icon = Icons.Outlined.NoPhotography,
			title = stringResource(R.string.qr_scanner_permission_denied_title),
			body = stringResource(R.string.qr_scanner_permission_denied_body),
			actionLabel = stringResource(R.string.qr_scanner_open_settings),
			onAction = onOpenSettings,
			secondaryActionLabel = fallbackRes?.let { stringResource(it) },
			onSecondaryAction = onFallback,
			modifier = modifier,
		)
	} else {
		QrScannerMessage(
			icon = Icons.Outlined.PhotoCamera,
			title = stringResource(R.string.qr_scanner_permission_title),
			body = stringResource(R.string.qr_scanner_permission_body),
			actionLabel = stringResource(R.string.qr_scanner_permission_allow),
			onAction = onRequestPermission,
			secondaryActionLabel = fallbackRes?.let { stringResource(it) },
			onSecondaryAction = onFallback,
			modifier = modifier,
		)
	}
}
