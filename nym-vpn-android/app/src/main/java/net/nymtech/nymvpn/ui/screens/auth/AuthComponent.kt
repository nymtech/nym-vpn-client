package net.nymtech.nymvpn.ui.screens.auth

import androidx.compose.animation.AnimatedContentTransitionScope
import androidx.compose.animation.core.tween
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.wrapContentHeight
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.navigation.compose.NavHost
import androidx.navigation.compose.composable
import androidx.navigation.compose.rememberNavController
import net.nymtech.nymvpn.ui.Route
import net.nymtech.nymvpn.ui.common.navigation.LocalNavController
import net.nymtech.nymvpn.ui.screens.account.create.modal.ExistingSubscriptionModal
import net.nymtech.nymvpn.ui.screens.auth.components.LoginView
import net.nymtech.nymvpn.ui.screens.auth.components.PassphraseView
import net.nymtech.nymvpn.ui.screens.auth.components.SignUpView
import net.nymtech.nymvpn.ui.screens.auth.components.WelcomeView
import net.nymtech.nymvpn.util.extensions.openWebUrl

@Composable
fun AuthComponent(
	initialRoute: AuthRoute,
	onAuthSuccess: () -> Unit,
	onSaveToPasswordManager: (passphrase: String) -> Unit,
	modifier: Modifier = Modifier,
	viewModel: AuthViewModel = hiltViewModel(),
) {
	val context = LocalContext.current

	val localNavController = rememberNavController()
	val rootNavController = LocalNavController.current

	val uiState by viewModel.uiState.collectAsStateWithLifecycle()

	LaunchedEffect(Unit) {
		viewModel.events.collect { event ->
			when (event) {
				is AuthEvent.SaveToPasswordManager -> {
					onSaveToPasswordManager(event.phrase)
				}
				is AuthEvent.LoginSuccess -> {
					onAuthSuccess()
					if (event.showTechnicalOpt) {
						rootNavController.navigate(Route.Technical)
					} else {
						onAuthSuccess()
					}
				}
				is AuthEvent.NavigateToGenerating -> {
					if (!uiState.isBillingAvailable && uiState.signUpLink != null) {
						context.openWebUrl(uiState.signUpLink!!)
						localNavController.navigate(AuthRoute.Login) {
							popUpTo(AuthRoute.Welcome)
						}
					} else {
						onAuthSuccess()
						rootNavController.navigate(Route.Generating())
					}
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
			.wrapContentHeight(),
		enterTransition = { slideIntoContainer(AnimatedContentTransitionScope.SlideDirection.Left, tween(300)) },
		exitTransition = { slideOutOfContainer(AnimatedContentTransitionScope.SlideDirection.Left, tween(300)) },
		popEnterTransition = { slideIntoContainer(AnimatedContentTransitionScope.SlideDirection.Right, tween(300)) },
		popExitTransition = { slideOutOfContainer(AnimatedContentTransitionScope.SlideDirection.Right, tween(300)) },
	) {
		composable<AuthRoute.Welcome> {
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
	}
}
