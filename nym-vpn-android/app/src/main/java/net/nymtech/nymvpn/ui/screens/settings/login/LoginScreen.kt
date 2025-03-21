package net.nymtech.nymvpn.ui.screens.settings.login

import android.Manifest
import android.view.WindowManager
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.asPaddingValues
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.systemBars
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.AppUiState
import net.nymtech.nymvpn.ui.AppViewModel
import net.nymtech.nymvpn.ui.MainActivity
import net.nymtech.nymvpn.ui.Route
import net.nymtech.nymvpn.ui.common.functions.rememberImeState
import net.nymtech.nymvpn.ui.common.navigation.LocalNavController
import net.nymtech.nymvpn.ui.common.navigation.NavBarState
import net.nymtech.nymvpn.ui.common.snackbar.SnackbarController
import net.nymtech.nymvpn.ui.screens.settings.login.components.LoginHeader
import net.nymtech.nymvpn.ui.screens.settings.login.components.LoginInputSection
import net.nymtech.nymvpn.ui.screens.settings.login.components.MaxDevicesModal
import net.nymtech.nymvpn.util.extensions.navigateAndForget
import net.nymtech.nymvpn.util.extensions.scaledHeight
import net.nymtech.nymvpn.util.extensions.scaledWidth

@Composable
fun LoginScreen(appUiState: AppUiState, appViewModel: AppViewModel, viewModel: LoginViewModel = hiltViewModel()) {
	val snackbar = SnackbarController.current
	val imeState = rememberImeState()
	val scrollState = rememberScrollState()
	val padding = WindowInsets.systemBars.asPaddingValues()
	val context = LocalContext.current
	val navController = LocalNavController.current

	val success by viewModel.success.collectAsStateWithLifecycle(null)
	val showMaxDevicesModal by viewModel.showMaxDevicesModal.collectAsStateWithLifecycle(null)
	var showModal by remember { mutableStateOf(false) }
	var loading by remember { mutableStateOf(false) }

	val activity = context as? MainActivity

	// Secure screen due to sensitive information
	DisposableEffect(Unit) {
		activity?.window?.setFlags(
			WindowManager.LayoutParams.FLAG_SECURE,
			WindowManager.LayoutParams.FLAG_SECURE,
		)
		onDispose {
			activity?.window?.clearFlags(WindowManager.LayoutParams.FLAG_SECURE)
		}
	}

	LaunchedEffect(success) {
		loading = false
		if (success == true) navController.navigateAndForget(Route.Main())
		if (success == false && showMaxDevicesModal == true) showModal = true
	}

	LaunchedEffect(Unit) {
		appViewModel.onNavBarStateChange(NavBarState(show = false))
	}

	LaunchedEffect(imeState.value) {
		if (imeState.value) {
			scrollState.animateScrollTo(scrollState.viewportSize)
		}
	}

	val requestPermissionLauncher = rememberLauncherForActivityResult(
		ActivityResultContracts.RequestPermission(),
	) { isGranted ->
		if (!isGranted) return@rememberLauncherForActivityResult snackbar.showMessage(context.getString(R.string.permission_required))
		navController.navigate(Route.LoginScanner)
	}

	Column(
		horizontalAlignment = Alignment.CenterHorizontally,
		verticalArrangement = Arrangement.spacedBy(40.dp.scaledHeight(), Alignment.Bottom),
		modifier = Modifier
			.fillMaxSize()
			.imePadding()
			.verticalScroll(scrollState)
			.padding(horizontal = 24.dp.scaledWidth())
			.padding(padding),
	) {
		LoginHeader()
		LoginInputSection(
			appUiState = appUiState,
			viewModel = viewModel,
			success = success,
			loading = loading,
			onLoadingChange = { loading = it },
			onRequestCameraPermission = { requestPermissionLauncher.launch(Manifest.permission.CAMERA) },
		)
	}

	MaxDevicesModal(
		show = showModal,
		accountLinks = appUiState.managerState.accountLinks,
		onDismiss = { showModal = false },
	)
}
