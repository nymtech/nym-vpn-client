package net.nymtech.nymvpn.ui.screens.main.bottomsheet.auth

import androidx.compose.animation.AnimatedContentTransitionScope
import androidx.compose.animation.animateContentSize
import androidx.compose.animation.core.tween
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.wrapContentHeight
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.rememberNavController
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.ui.AppUiState
import net.nymtech.nymvpn.ui.AuthRoute
import net.nymtech.nymvpn.ui.Route
import net.nymtech.nymvpn.ui.common.navigation.LocalNavController
import net.nymtech.nymvpn.ui.screens.main.bottomsheet.auth.modal.ExistingSubscriptionModal
import net.nymtech.nymvpn.ui.screens.main.bottomsheet.auth.components.LoginView
import net.nymtech.nymvpn.ui.screens.main.bottomsheet.auth.components.PassphraseView
import net.nymtech.nymvpn.ui.screens.main.bottomsheet.auth.components.SignUpView
import net.nymtech.nymvpn.ui.screens.main.bottomsheet.auth.components.TechOptView
import net.nymtech.nymvpn.ui.screens.main.bottomsheet.auth.components.WelcomeView
import net.nymtech.nymvpn.util.extensions.openWebUrl
import net.nymtech.nymvpn.util.extensions.savePasswordToManager

@Composable
fun AuthComponent(
	initialRoute: AuthRoute,
	onAuthSuccess: () -> Unit,
	onLoginProcessingStart: (passphrase: String) -> Unit,
	onWelcomeShown: () -> Unit = {},
	appUiState: AppUiState,
	modifier: Modifier = Modifier,
	viewModel: AuthViewModel = hiltViewModel(),
) {
	val context = LocalContext.current
	val coroutineScope = rememberCoroutineScope()

	val localNavController = rememberNavController()
	val rootNavController = LocalNavController.current

	val uiState by viewModel.uiState.collectAsStateWithLifecycle()
	val currentOnWelcomeShown by rememberUpdatedState(onWelcomeShown)

	LaunchedEffect(Unit) {
		viewModel.events.collect { event ->
			when (event) {
				is AuthEvent.LoginMnemonicImported -> {
					onLoginProcessingStart(event.phrase)
					coroutineScope.launch { savePasswordToManager(context = context, password = event.phrase) }
				}
				is AuthEvent.NavigateToGenerating -> {
					onAuthSuccess()
					rootNavController.navigate(Route.Generating())
				}
			}
		}
	}

	ExistingSubscriptionModal(
		showSubscriptionDialog = uiState.showExistingSubscriptionModal,
		onClickLogin = {
			viewModel.dismissSubscriptionModal()
			localNavController.navigate(AuthRoute.Login)
		},
		onClickCancel = { viewModel.dismissSubscriptionModal() },
		onDismiss = { viewModel.dismissSubscriptionModal() },
	)

	NavHost(
		navController = localNavController,
		startDestination = initialRoute,
		modifier = modifier
			.fillMaxWidth()
			.wrapContentHeight()
			.animateContentSize(),
		enterTransition = { slideIntoContainer(AnimatedContentTransitionScope.SlideDirection.Left, tween(300)) },
		exitTransition = { slideOutOfContainer(AnimatedContentTransitionScope.SlideDirection.Left, tween(300)) },
		popEnterTransition = { slideIntoContainer(AnimatedContentTransitionScope.SlideDirection.Right, tween(300)) },
		popExitTransition = { slideOutOfContainer(AnimatedContentTransitionScope.SlideDirection.Right, tween(300)) },
	) {
		composable<AuthRoute.Welcome> {
			LaunchedEffect(Unit) { currentOnWelcomeShown() }
			WelcomeView(
				onLoginClick = { localNavController.navigate(AuthRoute.Login) },
				onSignUpClick = { localNavController.navigate(AuthRoute.SignUp) },
			)
		}

		composable<AuthRoute.Login> {
			LoginView(
				onBackClick = { localNavController.popBackStack() },
				onSocialClick = { uiState.socialLink?.let { context.openWebUrl(it) } },
				onPassphraseClick = { localNavController.navigate(AuthRoute.Passphrase) },
			)
		}

		composable<AuthRoute.SignUp> {
			SignUpView(
				onBackClick = { localNavController.popBackStack() },
				onSocialClick = { uiState.socialLink?.let { context.openWebUrl(it) } },
				onAccountClick = { viewModel.onAnonymousAccountClick() },
			)
		}

		composable<AuthRoute.Passphrase> {
			PassphraseView(
				onBackClick = { localNavController.popBackStack() },
				mnemonicError = uiState.mnemonicError,
				loading = uiState.isLoading,
				mnemonic = uiState.mnemonic,
				onMnemonicChange = viewModel::onMnemonicChange,
				onSubmitMnemonic = viewModel::onSubmitMnemonic,
			)
		}

		composable<AuthRoute.TechOpt> {
			TechOptView(
				statsEnabled = appUiState.settings.statsEnabled,
				sentryEnabled = appUiState.vpnConfig.sentry,
				onNetworkStatsEnable = viewModel::onNetworkStatsEnabled,
				onMonitoringEnable = viewModel::onMonitoringEnabled,
				onContinueClick = {
					viewModel.onContinueClicked()
					onAuthSuccess()
				},
			)
		}
	}
}
