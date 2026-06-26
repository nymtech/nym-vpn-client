package net.nymtech.nymvpn.ui.screens.account.scanner

import android.Manifest
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.text.KeyboardOptions
import androidx.compose.material3.*
import androidx.compose.runtime.*
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.input.ImeAction
import androidx.compose.ui.unit.dp
import androidx.compose.ui.viewinterop.AndroidView
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.LifecycleEventObserver
import androidx.lifecycle.compose.LocalLifecycleOwner
import com.google.accompanist.permissions.ExperimentalPermissionsApi
import com.google.accompanist.permissions.isGranted
import com.google.accompanist.permissions.rememberPermissionState
import com.google.accompanist.permissions.shouldShowRationale
import com.google.zxing.BarcodeFormat
import com.journeyapps.barcodescanner.BarcodeCallback
import com.journeyapps.barcodescanner.BarcodeResult
import com.journeyapps.barcodescanner.DecoratedBarcodeView
import com.journeyapps.barcodescanner.DefaultDecoderFactory
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.Route
import net.nymtech.nymvpn.ui.common.navigation.LocalNavController
import net.nymtech.nymvpn.ui.screens.account.generating.GeneratingMode
import net.nymtech.nymvpn.util.FreepassParseResult
import net.nymtech.nymvpn.util.parseFreepassCode

@OptIn(ExperimentalPermissionsApi::class)
@Composable
fun FreepassScannerScreen(existingAccount: Boolean = false) {
    val navController = LocalNavController.current
    val cameraPermission = rememberPermissionState(Manifest.permission.CAMERA)
    var handled by rememberSaveable { mutableStateOf(false) }
    var manualInput by rememberSaveable { mutableStateOf("") }
    var manualError by remember { mutableStateOf(false) }

    fun proceed(code: String) {
        if (handled) return
        handled = true
        val destination = if (existingAccount) {
            Route.RedeemVoucher(code = code)
        } else {
            Route.Generating(mode = GeneratingMode.Freepass.name, code = code)
        }
        navController.navigate(destination) {
            popUpTo(Route.FreepassScanner()) { inclusive = true }
        }
    }

    fun onDecoded(raw: String) {
        when (val r = parseFreepassCode(raw)) {
            is FreepassParseResult.Valid -> proceed(r.code)
            FreepassParseResult.Invalid -> { /* ignore: keep scanning */ }
        }
    }

    Column(
        modifier = Modifier.fillMaxSize().padding(16.dp),
        horizontalAlignment = Alignment.CenterHorizontally,
    ) {
        // Title removed (sat too close to the system bar) but its vertical space is kept.
        Spacer(Modifier.height(24.dp))
        Text(stringResource(R.string.freepass_scanner_instruction), style = MaterialTheme.typography.bodyMedium)
        Spacer(Modifier.height(16.dp))

        if (cameraPermission.status.isGranted) {
            CameraScanner(onDecoded = ::onDecoded, modifier = Modifier.fillMaxWidth().weight(1f))
        } else {
            Column(Modifier.fillMaxWidth().weight(1f), horizontalAlignment = Alignment.CenterHorizontally) {
                Text(stringResource(R.string.freepass_scanner_camera_rationale))
                Spacer(Modifier.height(8.dp))
                if (cameraPermission.status.shouldShowRationale) {
                    Button(onClick = { cameraPermission.launchPermissionRequest() }) { Text(stringResource(R.string.freepass_scanner_open_settings)) }
                } else {
                    LaunchedEffect(Unit) { cameraPermission.launchPermissionRequest() }
                }
            }
        }

        Spacer(Modifier.height(16.dp))
        Text(stringResource(R.string.freepass_scanner_manual_label), style = MaterialTheme.typography.labelLarge)
        OutlinedTextField(
            value = manualInput,
            onValueChange = { manualInput = it; manualError = false },
            singleLine = true,
            isError = manualError,
            label = { Text(stringResource(R.string.freepass_scanner_manual_hint)) },
            keyboardOptions = KeyboardOptions(imeAction = ImeAction.Done),
            modifier = Modifier.fillMaxWidth(),
        )
        if (manualError) {
            Text(stringResource(R.string.freepass_scanner_invalid_input), color = MaterialTheme.colorScheme.error, style = MaterialTheme.typography.bodySmall)
        }
        Spacer(Modifier.height(8.dp))
        Button(
            onClick = {
                when (val r = parseFreepassCode(manualInput)) {
                    is FreepassParseResult.Valid -> proceed(r.code)
                    FreepassParseResult.Invalid -> manualError = true
                }
            },
            modifier = Modifier.fillMaxWidth(),
        ) { Text(stringResource(R.string.freepass_scanner_submit)) }
    }
}

@Composable
private fun CameraScanner(onDecoded: (String) -> Unit, modifier: Modifier = Modifier) {
    val lifecycleOwner = LocalLifecycleOwner.current
    val currentOnDecoded by rememberUpdatedState(onDecoded)
    // Renamed to scannerViewRef to avoid clash with DecoratedBarcodeView.getBarcodeView()
    val scannerViewRef = remember { mutableStateOf<DecoratedBarcodeView?>(null) }

    AndroidView(
        modifier = modifier,
        factory = { context ->
            DecoratedBarcodeView(context).apply {
                // setDecoderFactory is on DecoratedBarcodeView directly in 4.3.0
                setDecoderFactory(DefaultDecoderFactory(listOf(BarcodeFormat.QR_CODE)))
                setStatusText("")
                decodeContinuous(object : BarcodeCallback {
                    override fun barcodeResult(result: BarcodeResult) {
                        result.text?.let { currentOnDecoded(it) }
                    }
                })
                scannerViewRef.value = this
            }
        },
    )

    DisposableEffect(lifecycleOwner) {
        val observer = LifecycleEventObserver { _, event ->
            when (event) {
                Lifecycle.Event.ON_RESUME -> scannerViewRef.value?.resume()
                Lifecycle.Event.ON_PAUSE -> scannerViewRef.value?.pause()
                else -> Unit
            }
        }
        lifecycleOwner.lifecycle.addObserver(observer)
        onDispose {
            lifecycleOwner.lifecycle.removeObserver(observer)
            scannerViewRef.value?.pause()
        }
    }
}
