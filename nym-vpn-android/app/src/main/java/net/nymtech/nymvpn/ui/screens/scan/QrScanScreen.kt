package net.nymtech.nymvpn.ui.screens.scan

import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.foundation.layout.padding
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import net.nymtech.nymvpn.ui.screens.scan.components.ConnectAfterCheckbox
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.common.functions.SecureScreen
import net.nymtech.nymvpn.ui.common.navigation.LocalNavController
import net.nymtech.nymvpn.ui.common.security.SecureOverlayProtection
import net.nymtech.qrscanner.QrScanner
import net.nymtech.qrscanner.model.QrScanAction
import net.nymtech.qrscanner.model.QrScanMode

@Composable
fun QrScanScreen(fromWelcome: Boolean, accountStored: Boolean, viewModel: QrScanViewModel = hiltViewModel(), redeemViewModel: RedeemCodeViewModel = hiltViewModel()) {
	SecureScreen()
	SecureOverlayProtection()

	val navController = LocalNavController.current
	val hadAccount = rememberSaveable { accountStored }
	val openAcceptedCode = rememberOpenAcceptedCode(hadAccount)
	var showRedeemSheet by rememberSaveable { mutableStateOf(false) }
	// Shared with the redeem sheet, which uses the same view model.
	val redeemUiState by redeemViewModel.uiState.collectAsStateWithLifecycle()

	QrScanner(
		mode = QrScanMode.ANY,
		validator = viewModel,
		onResult = { result -> ScannedQrCode.parse(result.contents)?.let { openAcceptedCode(it, redeemUiState.connectAfter) } },
		onClose = { navController.leaveCodeEntry(fromWelcome) },
		secondaryAction = QrScanAction(stringResource(R.string.redeem_code_enter_instead)) { showRedeemSheet = true },
		paused = showRedeemSheet,
		bottomContent = { enabled ->
			ConnectAfterCheckbox(
				checked = redeemUiState.connectAfter,
				onCheckedChange = redeemViewModel::onConnectAfterChange,
				label = stringResource(R.string.qr_scan_connect_when_done),
				enabled = enabled,
				textColor = Color.White,
				modifier = Modifier.padding(bottom = 8.dp),
			)
		},
	)

	if (showRedeemSheet) {
		RedeemCodeSheet(accountStored = hadAccount, onDismiss = { showRedeemSheet = false }, viewModel = redeemViewModel)
	}
}
