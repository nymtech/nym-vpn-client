package net.nymtech.nymvpn.ui.screens.scan

import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import androidx.navigation.NavController
import androidx.navigation.NavDestination.Companion.hasRoute
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.AuthRoute
import net.nymtech.nymvpn.ui.Route
import net.nymtech.nymvpn.ui.common.navigation.LocalNavController
import net.nymtech.nymvpn.ui.common.snackbar.AlertController
import net.nymtech.nymvpn.ui.common.snackbar.AlertMessage
import net.nymtech.nymvpn.ui.common.snackbar.AlertType
import net.nymtech.nymvpn.ui.routeName
import net.nymtech.nymvpn.util.extensions.navigateAndForgetToMain
import net.nymtech.nymvpn.util.extensions.replaceCurrentWith
import net.nymtech.nymvpn.util.extensions.safePopBackStack

// The validator has already acted on the code.
@Composable
fun rememberOpenAcceptedCode(): (ScannedQrCode) -> Unit {
	val navController = LocalNavController.current
	val freepassActivated = stringResource(R.string.freepass_activated)
	return { code ->
		when (code) {
			// From a sheet, the host screen stays below.
			is ScannedQrCode.Pairing -> if (navController.currentDestination?.hasRoute<Route.QrScan>() == true) {
				navController.replaceCurrentWith(Route.PairingSignIn)
			} else {
				navController.navigate(Route.PairingSignIn)
			}
			is ScannedQrCode.Freepass -> {
				AlertController.show(AlertMessage(type = AlertType.Confirmation, title = freepassActivated))
				// Login processing waits for device registration and zk-nyms.
				navController.navigateAndForgetToMain(Route.Main(autoStart = false, loginProcessing = true))
			}
		}
	}
}

fun NavController.leaveCodeEntry(fromWelcome: Boolean) {
	// The welcome sheet doesn't survive leaving Main, so reopen it.
	if (fromWelcome) navigateAndForgetToMain(Route.Main(authRoute = AuthRoute.Welcome.routeName)) else safePopBackStack()
}
