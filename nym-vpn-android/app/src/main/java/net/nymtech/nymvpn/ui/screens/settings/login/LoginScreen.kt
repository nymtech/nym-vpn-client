package net.nymtech.nymvpn.ui.screens.settings.login

import android.Manifest
import android.view.WindowManager
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
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
import net.nymtech.nymvpn.ui.MainActivity
import net.nymtech.nymvpn.ui.Route
import net.nymtech.nymvpn.ui.common.functions.rememberImeState
import net.nymtech.nymvpn.ui.common.navigation.LocalNavController
import net.nymtech.nymvpn.ui.common.snackbar.SnackbarController
import net.nymtech.nymvpn.ui.screens.settings.login.components.LoginHeader
import net.nymtech.nymvpn.ui.screens.settings.login.components.LoginInputSection
import net.nymtech.nymvpn.ui.screens.settings.login.components.MaxDevicesModal
import net.nymtech.nymvpn.util.extensions.replaceCurrentWith
import net.nymtech.nymvpn.util.extensions.scaledHeight
import net.nymtech.nymvpn.util.extensions.scaledWidth
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.LinkAnnotation
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextDecoration
import androidx.compose.ui.text.withLink
import androidx.compose.ui.text.withStyle

@Composable
fun LoginScreen(appUiState: AppUiState, viewModel: LoginViewModel = hiltViewModel()) {
	val snackbar = SnackbarController.current
	val imeState = rememberImeState()
	val scrollState = rememberScrollState()
	val context = LocalContext.current
	val navController = LocalNavController.current

	val uiState by viewModel.uiState.collectAsStateWithLifecycle()
	var loading by remember { mutableStateOf(false) }

	val activity = context as? MainActivity

	DisposableEffect(Unit) {
		activity?.window?.setFlags(
			WindowManager.LayoutParams.FLAG_SECURE,
			WindowManager.LayoutParams.FLAG_SECURE,
		)
		onDispose {
			activity?.window?.clearFlags(WindowManager.LayoutParams.FLAG_SECURE)
		}
	}

	LaunchedEffect(uiState.success, uiState.showTechnicalOptScreen) {
		if (uiState.success != null) loading = false

		if (uiState.success == true) {
			if (uiState.showTechnicalOptScreen) {
				navController.replaceCurrentWith(Route.Technical)
				viewModel.consumeTechnicalOptFlag()
			} else {
				navController.replaceCurrentWith(Route.Main())
			}
			viewModel.consumeResult()
		} else if (uiState.success == false) {
			viewModel.consumeResult()
		}
	}

	LaunchedEffect(imeState.value) {
		if (imeState.value) {
			scrollState.animateScrollTo(scrollState.viewportSize)
		}
	}

	val permissionRequiredText =
		stringResource(id = R.string.permission_required)

	val requestPermissionLauncher = rememberLauncherForActivityResult(
		ActivityResultContracts.RequestPermission(),
	) { isGranted ->
		if (!isGranted) return@rememberLauncherForActivityResult snackbar.showMessage(permissionRequiredText)
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
			.navigationBarsPadding(),
	) {
		LoginHeader()
		LoginInputSection(
			onCreateAccountClick = {
				navController.navigate(Route.WelcomeAccount)
			},
			viewModel = viewModel,
			uiState = uiState,
			loading = loading,
			onLoadingChange = { loading = it },
			onRequestCameraPermission = { requestPermissionLauncher.launch(Manifest.permission.CAMERA) },
		)

		Text(
			text = buildAnnotatedString {
				append(stringResource(R.string.account_welcome_privacy_start))
				append("\n")
				withStyle(
					SpanStyle(
						color = MaterialTheme.colorScheme.onBackground,
						textDecoration = TextDecoration.Underline,
					),
				) {
					withLink(LinkAnnotation.Url(stringResource(R.string.terms_link))) {
						append(stringResource(R.string.terms_of_use))
					}
				}
				append(" ")
				append(stringResource(R.string.account_welcome_privacy_middle))
				append(" ")
				withStyle(
					SpanStyle(
						color = MaterialTheme.colorScheme.onBackground,
						textDecoration = TextDecoration.Underline,
					),
				) {
					withLink(LinkAnnotation.Url(stringResource(R.string.privacy_link))) {
						append(stringResource(R.string.privacy_policy))
					}
				}
				append(".")
			},
			textAlign = TextAlign.Center,
			style = MaterialTheme.typography.bodySmall.copy(
				color = MaterialTheme.colorScheme.outline,
				fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
			),
			modifier = Modifier
				.fillMaxWidth()
				.padding(top = 16.dp.scaledHeight()),
		)
	}

	MaxDevicesModal(
		show = uiState.showMaxDevicesModal,
		accountLinks = appUiState.managerState.accountLinks,
		onDismiss = { viewModel.dismissMaxDevicesModal() },
	)
}
