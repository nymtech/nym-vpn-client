package net.nymtech.nymvpn.ui.screens.account.pairing

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
fun PairingScanScreen(viewModel: PairingScanViewModel = hiltViewModel()) {
	SecureScreen()
	SecureOverlayProtection()

	val navController = LocalNavController.current

	QrScanner(
		mode = QrScanMode.PAIRING,
		validator = viewModel.validator,
		onResult = { result ->
			when (result) {
				// The validator has already started pairing with the scanned code.
				is QrScanResult.Pairing -> navController.replaceCurrentWith(Route.PairingSignIn)
				QrScanResult.Fallback -> navController.navigateAndForgetToMain(Route.Main(authRoute = AuthRoute.Passphrase.routeName))
			}
		},
		// The login sheet doesn't survive leaving Main, so reopen it where the user was.
		onClose = { navController.navigateAndForgetToMain(Route.Main(authRoute = AuthRoute.Login.routeName)) },
	)
}
