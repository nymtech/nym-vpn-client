package net.nymtech.nymvpn.ui.screens.settings.login

import PrivacyText
import android.content.res.Configuration
import android.view.WindowManager
import androidx.compose.foundation.ScrollState
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
import androidx.lifecycle.Lifecycle
import androidx.lifecycle.compose.LocalLifecycleOwner
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.lifecycle.lifecycleScope
import androidx.lifecycle.repeatOnLifecycle
import kotlinx.coroutines.flow.collectLatest
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.AppUiState
import net.nymtech.nymvpn.ui.AppViewModel
import net.nymtech.nymvpn.ui.MainActivity
import net.nymtech.nymvpn.ui.Route
import net.nymtech.nymvpn.ui.Route.*
import net.nymtech.nymvpn.ui.common.functions.rememberImeState
import net.nymtech.nymvpn.ui.common.navigation.LocalNavController
import net.nymtech.nymvpn.ui.screens.settings.login.components.LoginInputSection
import net.nymtech.nymvpn.ui.screens.settings.login.components.MaxDevicesModal
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.util.extensions.openWebUrl
import net.nymtech.nymvpn.util.extensions.savePasswordToManager
import net.nymtech.nymvpn.util.extensions.scaledWidth

@Composable
fun LoginScreen(appUiState: AppUiState, appViewModel: AppViewModel, viewModel: LoginViewModel = hiltViewModel()) {
	val imeState = rememberImeState()
	val scrollState = rememberScrollState()
	val context = LocalContext.current
	val navController = LocalNavController.current
	val lifecycleOwner = LocalLifecycleOwner.current

	val uiState by viewModel.uiState.collectAsStateWithLifecycle()
	val activity = context as? MainActivity

	DisposableEffect(Unit) {
		activity?.window?.setFlags(
			WindowManager.LayoutParams.FLAG_SECURE,
			WindowManager.LayoutParams.FLAG_SECURE,
		)
		onDispose { activity?.window?.clearFlags(WindowManager.LayoutParams.FLAG_SECURE) }
	}

	LaunchedEffect(lifecycleOwner) {
		lifecycleOwner.repeatOnLifecycle(Lifecycle.State.STARTED) {
			viewModel.events.collectLatest { event ->
				when (event) {
					is LoginEvent.NavigateAfterLogin -> {
						appViewModel.notifyLoginStarted()
					}
					LoginEvent.Processing -> {
						val pass = uiState.mnemonic.trim()
						if (pass.isNotEmpty()) {
							activity?.lifecycleScope?.launch {
								savePasswordToManager(context = context, password = pass)
							}
						}
					}
				}
			}
		}
	}

	LaunchedEffect(imeState.value) {
		if (imeState.value) scrollState.animateScrollTo(scrollState.viewportSize)
	}

	LoginScreen(
		scrollState = scrollState,
		uiState = uiState,
		onMnemonicChange = viewModel::onMnemonicChange,
		onSubmitMnemonic = {
			viewModel.onSubmitMnemonic()
		},
		onCreateAccountClick = { navController.navigate(Route.CreateAccount) },
		onSocialClick = { uiState.deeplink?.let { context.openWebUrl(it) } },
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
	onMnemonicChange: (String) -> Unit,
	onSubmitMnemonic: () -> Unit,
	onCreateAccountClick: () -> Unit,
	onSocialClick: () -> Unit,
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
					onSocialClick = onSocialClick,
					mnemonicError = uiState.mnemonicError,
					loading = uiState.isLoading,
					mnemonic = uiState.mnemonic,
					onMnemonicChange = onMnemonicChange,
					onSubmitMnemonic = { onSubmitMnemonic() },
				)
			}

			Spacer(modifier = Modifier.weight(1f))

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
			onMnemonicChange = {},
			onSubmitMnemonic = {},
			onCreateAccountClick = { },
			onSocialClick = { },
		)
	}
}
