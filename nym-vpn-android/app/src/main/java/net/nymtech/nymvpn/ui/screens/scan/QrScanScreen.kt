package net.nymtech.nymvpn.ui.screens.scan

import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.res.stringResource
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.common.functions.SecureScreen
import net.nymtech.nymvpn.ui.common.navigation.LocalNavController
import net.nymtech.nymvpn.ui.common.security.SecureOverlayProtection
import net.nymtech.qrscanner.QrScanner
import net.nymtech.qrscanner.model.QrScanAction
import net.nymtech.qrscanner.model.QrScanMode

@Composable
fun QrScanScreen(fromWelcome: Boolean, viewModel: QrScanViewModel = hiltViewModel()) {
	SecureScreen()
	SecureOverlayProtection()

	val navController = LocalNavController.current
	val openAcceptedCode = rememberOpenAcceptedCode()
	var showRedeemSheet by rememberSaveable { mutableStateOf(false) }

	QrScanner(
		mode = QrScanMode.ANY,
		validator = viewModel,
		onResult = { result -> ScannedQrCode.parse(result.contents)?.let(openAcceptedCode) },
		onClose = { navController.leaveCodeEntry(fromWelcome) },
		secondaryAction = QrScanAction(stringResource(R.string.redeem_code_enter_instead)) { showRedeemSheet = true },
		paused = showRedeemSheet,
	)

	if (showRedeemSheet) {
		RedeemCodeSheet(onDismiss = { showRedeemSheet = false })
	}
}
