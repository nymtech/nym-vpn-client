package net.nymtech.nymvpn.ui.screens.scanner

import android.app.Activity
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.remember
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.viewinterop.AndroidView
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import com.journeyapps.barcodescanner.CompoundBarcodeView
import net.nymtech.nymvpn.ui.Route
import net.nymtech.nymvpn.ui.common.navigation.LocalNavController
import net.nymtech.nymvpn.util.extensions.navigateAndForget

@Composable
fun ScannerScreen(viewModel: ScannerViewModel = hiltViewModel()) {
	val context = LocalContext.current
	val navController = LocalNavController.current

	val success = viewModel.success.collectAsStateWithLifecycle(null)

	LaunchedEffect(success.value) {
		if (success.value == true) navController.navigateAndForget(Route.Main())
		if (success.value == false) navController.popBackStack()
	}

	val barcodeView = remember {
		CompoundBarcodeView(context).apply {
			this.initializeFromIntent((context as Activity).intent)
			this.setStatusText("")
			this.decodeSingle { result ->
				result.text?.let { barCodeOrQr ->
					viewModel.onMnemonicImport(barCodeOrQr)
				}
			}
		}
	}
	AndroidView(factory = { barcodeView })
	DisposableEffect(Unit) {
		barcodeView.resume()
		onDispose {
			barcodeView.pause()
		}
	}
}
