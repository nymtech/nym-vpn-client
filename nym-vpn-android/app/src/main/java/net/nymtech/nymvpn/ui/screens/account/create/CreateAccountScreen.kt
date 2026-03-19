package net.nymtech.nymvpn.ui.screens.account.create

import PrivacyText
import android.content.res.Configuration
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Lock
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
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
import net.nymtech.nymvpn.ui.Route
import net.nymtech.nymvpn.ui.common.animations.SpinningIcon
import net.nymtech.nymvpn.ui.common.buttons.MainStyledButton
import net.nymtech.nymvpn.ui.common.buttons.OutlineStyledButton
import net.nymtech.nymvpn.ui.common.navigation.LocalNavController
import net.nymtech.nymvpn.ui.screens.account.create.components.CreateAccountBlock
import net.nymtech.nymvpn.ui.screens.account.create.modal.ExistingSubscriptionModal
import net.nymtech.nymvpn.ui.theme.CustomTypography
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.util.extensions.navigateAndForget
import net.nymtech.nymvpn.util.extensions.openWebUrl
import net.nymtech.nymvpn.util.extensions.replaceCurrentWith
import net.nymtech.nymvpn.util.extensions.scaledHeight
import net.nymtech.nymvpn.util.extensions.scaledWidth

@Composable
fun CreateAccountScreen(appUiState: AppUiState, viewModel: CreateAccountViewModel = hiltViewModel()) {
	val context = LocalContext.current
	val navController = LocalNavController.current

	val ui by viewModel.uiState.collectAsStateWithLifecycle()
	var showSubscriptionDialog by remember { mutableStateOf(false) }

	CreateAccountScreen(
		loading = ui.isLoading,
		onLogInClick = { navController.navigateAndForget(Route.Login) },
		onCreateAccountClick = {
			if (ui.isLoading) return@CreateAccountScreen

			if (ui.isBillingAvailable) {
				if (ui.hasActiveSubscription) {
					showSubscriptionDialog = true
				} else {
					navController.replaceCurrentWith(Route.Generating())
				}
			} else {
				ui.signUpLink?.let { context.openWebUrl(it) }
				navController.replaceCurrentWith(Route.Login)
			}
		},
		onSocialClick = {
			ui.deeplink?.let {
				context.openWebUrl(it)
			}
		},
	)

	ExistingSubscriptionModal(
		showSubscriptionDialog = showSubscriptionDialog,
		onClickLogin = {
			showSubscriptionDialog = false
			navController.navigateAndForget(Route.Login)
		},
		onClickCancel = { showSubscriptionDialog = false },
		onDismiss = { showSubscriptionDialog = false },
	)
}

@Composable
fun CreateAccountScreen(loading: Boolean, onLogInClick: () -> Unit, onCreateAccountClick: () -> Unit, onSocialClick: () -> Unit) {
	Column(
		modifier = Modifier
			.fillMaxSize()
			.background(MaterialTheme.colorScheme.background)
			.imePadding()
			.padding(horizontal = 24.dp.scaledWidth())
			.navigationBarsPadding(),
	) {
		Column(
			horizontalAlignment = Alignment.CenterHorizontally,
			modifier = Modifier
				.weight(1f)
				.fillMaxWidth(),
		) {
			Spacer(modifier = Modifier.weight(1f))

			Text(
				text = stringResource(R.string.onboarding_create_account_button),
				style = MaterialTheme.typography.headlineSmall,
				color = MaterialTheme.colorScheme.onBackground,
				textAlign = TextAlign.Center,
				fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
				modifier = Modifier.fillMaxWidth(),
			)

			Spacer(modifier = Modifier.height(100.dp.scaledHeight()))

			Column(
				horizontalAlignment = Alignment.CenterHorizontally,
				modifier = Modifier.fillMaxWidth(),
			) {
				CreateAccountBlock(
					title = stringResource(R.string.account_welcome_create_title),
					description = stringResource(R.string.account_welcome_create_description),
					button = {
						MainStyledButton(
							onClick = onCreateAccountClick,
							content = {
								if (loading) {
									SpinningIcon(Icons.Outlined.Lock, "")
								} else {
									Text(
										text = stringResource(R.string.account_welcome_create_button),
										style = CustomTypography.buttonMain,
									)
								}
							},
							color = MaterialTheme.colorScheme.primary,
							modifier = Modifier
								.fillMaxWidth()
								.height(54.dp.scaledHeight()),
						)
					},
				)
				Spacer(modifier = Modifier.height(24.dp.scaledHeight()))
				HorizontalDivider(color = MaterialTheme.colorScheme.outline.copy(alpha = 0.35f))
				Spacer(modifier = Modifier.height(24.dp.scaledHeight()))
				CreateAccountBlock(
					title = stringResource(R.string.account_welcome_social_title),
					description = stringResource(R.string.account_welcome_social_description),
					button = {
						OutlineStyledButton(
							onClick = onSocialClick,
							content = {
								if (loading) {
									SpinningIcon(Icons.Outlined.Lock, "")
								} else {
									Text(
										text = stringResource(R.string.account_welcome_social_button),
										style = CustomTypography.buttonMain,
									)
								}
							},
							borderColor = MaterialTheme.colorScheme.onBackground,
							modifier = Modifier
								.fillMaxWidth()
								.height(54.dp.scaledHeight()),
						)
					},
				)
				Spacer(modifier = Modifier.height(24.dp.scaledHeight()))
				HorizontalDivider(color = MaterialTheme.colorScheme.outline.copy(alpha = 0.35f))
				Spacer(modifier = Modifier.height(24.dp.scaledHeight()))

				Row(
					modifier = Modifier.fillMaxWidth(),
					horizontalArrangement = Arrangement.Center,
					verticalAlignment = Alignment.CenterVertically,
				) {
					Text(
						text = stringResource(R.string.account_welcome_login_title),
						style = MaterialTheme.typography.labelLarge,
						color = MaterialTheme.colorScheme.outline,
						fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
					)
					Spacer(modifier = Modifier.width(4.dp))
					Text(
						text = stringResource(R.string.log_in),
						style = MaterialTheme.typography.labelLarge,
						color = MaterialTheme.colorScheme.primary,
						fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
						modifier = Modifier.clickable { onLogInClick() },
					)
				}
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
internal fun PreviewCreateAccountScreen() {
	NymVPNTheme(Theme.default()) {
		CreateAccountScreen(
			loading = false,
			onLogInClick = {},
			onCreateAccountClick = {},
			onSocialClick = {},
		)
	}
}
