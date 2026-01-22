package net.nymtech.nymvpn.ui.screens.settings.login

import PrivacyText
import android.content.res.Configuration
import android.view.WindowManager
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.ScrollState
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
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
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.tooling.preview.Preview
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
import net.nymtech.nymvpn.ui.screens.settings.login.components.LoginInputSection
import net.nymtech.nymvpn.ui.screens.settings.login.components.MaxDevicesModal
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.util.extensions.replaceCurrentWith
import net.nymtech.nymvpn.util.extensions.scaledWidth

@Composable
fun LoginScreen(appUiState: AppUiState, viewModel: LoginViewModel = hiltViewModel()) {
	val snackbar = SnackbarController.current
	val imeState = rememberImeState()
	val scrollState = rememberScrollState()
	val context = LocalContext.current
	val navController = LocalNavController.current

	val uiState by viewModel.uiState.collectAsStateWithLifecycle()
	var loading by remember { mutableStateOf(false) }
	var mnemonic by remember { mutableStateOf("") }

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

	val permissionRequiredText = stringResource(id = R.string.permission_required)

	val requestPermissionLauncher = rememberLauncherForActivityResult(
		ActivityResultContracts.RequestPermission(),
	) { isGranted ->
		if (!isGranted) return@rememberLauncherForActivityResult snackbar.showMessage(permissionRequiredText)
		navController.navigate(Route.LoginScanner)
	}

	LoginScreen(
		scrollState = scrollState,
		uiState = uiState,
		loading = loading,
		mnemonic = mnemonic,
		onMnemonicChange = { mnemonic = it },
		onSubmitMnemonic = { phrase ->
			loading = true
			viewModel.onMnemonicImport(phrase)
		},
		onDismissError = { viewModel.consumeResult() },
		onCreateAccountClick = { navController.navigate(Route.CreateAccount) },
	)

	MaxDevicesModal(
		show = uiState.showMaxDevicesModal,
		accountLinks = appUiState.managerState.accountLinks,
		onDismiss = { viewModel.dismissMaxDevicesModal() },
	)
}

@Composable
private fun LoginScreen(
	scrollState: ScrollState,
	uiState: LoginUiState,
	loading: Boolean,
	mnemonic: String,
	onMnemonicChange: (String) -> Unit,
	onSubmitMnemonic: (String) -> Unit,
	onDismissError: () -> Unit,
	onCreateAccountClick: () -> Unit,
) {
	Column(
		horizontalAlignment = Alignment.CenterHorizontally,
		modifier = Modifier
			.fillMaxSize()
			.imePadding()
			.padding(horizontal = 24.dp.scaledWidth())
			.navigationBarsPadding(),
	) {
		Column(
			horizontalAlignment = Alignment.CenterHorizontally,
			modifier = Modifier
				.weight(1f)
				.fillMaxWidth()
				.verticalScroll(scrollState),
		) {
			Spacer(modifier = Modifier.weight(1f))

			Text(
				text = stringResource(R.string.log_in),
				style = MaterialTheme.typography.headlineSmall,
				color = MaterialTheme.colorScheme.onBackground,
				fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
			)

			Spacer(modifier = Modifier.height(100.dp))

			Column(
				horizontalAlignment = Alignment.CenterHorizontally,
				modifier = Modifier.fillMaxWidth(),
			) {
				Text(
					text = stringResource(R.string.enter_access_code),
					style = MaterialTheme.typography.bodyMedium,
					color = MaterialTheme.colorScheme.outline,
					fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
					textAlign = TextAlign.Start,
					modifier = Modifier.fillMaxWidth(),
				)

				Spacer(modifier = Modifier.height(24.dp))

				LoginInputSection(
					onCreateAccountClick = onCreateAccountClick,
					uiState = uiState,
					loading = loading,
					mnemonic = mnemonic,
					onMnemonicChange = onMnemonicChange,
					onSubmitMnemonic = onSubmitMnemonic,
					onDismissError = onDismissError,
				)
			}

			Spacer(modifier = Modifier.weight(1f))
		}

		Box(
			modifier = Modifier
				.fillMaxWidth()
				.padding(bottom = 24.dp),
			contentAlignment = Alignment.BottomCenter,
		) {
			PrivacyText()
		}
	}
}

@Preview(uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
internal fun PreviewLoginScreen() {
	NymVPNTheme(Theme.default()) {
		LoginScreen(
			scrollState = rememberScrollState(),
			uiState = LoginUiState(),
			loading = false,
			mnemonic = "",
			onMnemonicChange = {},
			onSubmitMnemonic = {},
			onDismissError = {},
			onCreateAccountClick = {},
		)
	}
}
