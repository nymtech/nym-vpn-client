package net.nymtech.nymvpn.ui.screens.scanner

import android.app.Activity
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.remember
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.viewinterop.AndroidView
import com.google.accompanist.permissions.ExperimentalPermissionsApi
import com.journeyapps.barcodescanner.CompoundBarcodeView
import net.nymtech.nymvpn.ui.AppViewModel

@OptIn(ExperimentalPermissionsApi::class)
@Composable
fun ScannerScreen(appViewModel: AppViewModel) {
	val context = LocalContext.current

	val barcodeView = remember {
		CompoundBarcodeView(context).apply {
			this.initializeFromIntent((context as Activity).intent)
			this.setStatusText("")
			this.decodeSingle { result ->
				result.text?.let { barCodeOrQr ->
					appViewModel.onCredentialImport(barCodeOrQr)
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
