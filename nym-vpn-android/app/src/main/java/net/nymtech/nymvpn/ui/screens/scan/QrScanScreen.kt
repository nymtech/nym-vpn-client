package net.nymtech.nymvpn.ui.screens.scan

import androidx.compose.runtime.Composable
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import net.nymtech.nymvpn.ui.AuthRoute
import net.nymtech.nymvpn.ui.Route
import net.nymtech.nymvpn.ui.common.functions.SecureScreen
import net.nymtech.nymvpn.ui.common.navigation.LocalNavController
import net.nymtech.nymvpn.ui.common.security.SecureOverlayProtection
import net.nymtech.nymvpn.ui.routeName
import net.nymtech.nymvpn.util.extensions.navigateAndForgetToMain
import net.nymtech.nymvpn.util.extensions.replaceCurrentWith
import net.nymtech.qrscanner.QrScanner
import net.nymtech.qrscanner.model.QrScanMode
import net.nymtech.qrscanner.model.QrScanResult

@Composable
fun QrScanScreen(mode: QrScanMode, returnTo: AuthRoute, viewModel: QrScanViewModel = hiltViewModel()) {
	SecureScreen()
	SecureOverlayProtection()

	val navController = LocalNavController.current

	QrScanner(
		mode = mode,
		validator = viewModel,
		onResult = { result ->
			when (result) {
				is QrScanResult.Code -> when (ScannedQrCode.parse(result.contents)) {
					// The validator has already started pairing with this code.
					is ScannedQrCode.Pairing -> navController.replaceCurrentWith(Route.PairingSignIn)
					// Not reachable: the validator rejects unknown codes.
					null -> Unit
				}
				QrScanResult.Fallback -> navController.navigateAndForgetToMain(Route.Main(authRoute = AuthRoute.Passphrase.routeName))
			}
		},
		// The auth sheet doesn't survive leaving Main, so reopen it where the user was.
		onClose = { navController.navigateAndForgetToMain(Route.Main(authRoute = returnTo.routeName)) },
	)
}
