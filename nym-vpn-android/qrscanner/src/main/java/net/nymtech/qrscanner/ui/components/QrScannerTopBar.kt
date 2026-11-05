package net.nymtech.qrscanner.ui.components

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.statusBarsPadding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Close
import androidx.compose.material.icons.outlined.FlashlightOff
import androidx.compose.material.icons.outlined.FlashlightOn
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.IconButtonDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import net.nymtech.qrscanner.R
import net.nymtech.qrscanner.ui.theme.QrScannerColors

@Composable
internal fun QrScannerTopBar(showTorch: Boolean, torchEnabled: Boolean, onClose: () -> Unit, onTorchToggle: () -> Unit, modifier: Modifier = Modifier) {
	Row(
		modifier = modifier
			.fillMaxWidth()
			.statusBarsPadding()
			.padding(horizontal = 16.dp, vertical = 8.dp),
		horizontalArrangement = Arrangement.SpaceBetween,
		verticalAlignment = Alignment.CenterVertically,
	) {
		OverlayIconButton(
			icon = Icons.Filled.Close,
			contentDescription = stringResource(R.string.qr_scanner_close),
			onClick = onClose,
		)
		if (showTorch) {
			OverlayIconButton(
				icon = if (torchEnabled) Icons.Outlined.FlashlightOn else Icons.Outlined.FlashlightOff,
				contentDescription = stringResource(if (torchEnabled) R.string.qr_scanner_torch_off else R.string.qr_scanner_torch_on),
				onClick = onTorchToggle,
				tint = if (torchEnabled) MaterialTheme.colorScheme.primary else QrScannerColors.onOverlay,
			)
		}
	}
}

@Composable
private fun OverlayIconButton(icon: ImageVector, contentDescription: String, onClick: () -> Unit, modifier: Modifier = Modifier, tint: Color = QrScannerColors.onOverlay) {
	IconButton(
		onClick = onClick,
		modifier = modifier,
		colors = IconButtonDefaults.iconButtonColors(containerColor = QrScannerColors.overlayButton),
	) {
		Icon(imageVector = icon, contentDescription = contentDescription, tint = tint)
	}
}
