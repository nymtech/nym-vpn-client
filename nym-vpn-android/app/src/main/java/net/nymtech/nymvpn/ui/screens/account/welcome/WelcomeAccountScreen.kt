package net.nymtech.nymvpn.ui.screens.account.welcome

import android.content.res.Configuration
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.outlined.OpenInNew
import androidx.compose.material.icons.outlined.Lock
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.ColorFilter
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.painterResource
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
import net.nymtech.nymvpn.ui.screens.account.welcome.modal.ExistingSubscriptionModal
import net.nymtech.nymvpn.ui.theme.CustomTypography
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.util.extensions.navigateAndForget
import net.nymtech.nymvpn.util.extensions.openWebUrl
import net.nymtech.nymvpn.util.extensions.replaceCurrentWith
import net.nymtech.nymvpn.util.extensions.scaledHeight
import net.nymtech.nymvpn.util.extensions.scaledWidth
import timber.log.Timber

@Composable
fun WelcomeAccountScreen(appUiState: AppUiState, viewModel: WelcomeAccountViewModel = hiltViewModel()) {
	val context = LocalContext.current
	val navController = LocalNavController.current

	val activeSubscription by viewModel.activeSubscription.collectAsStateWithLifecycle(false)
	val loading by viewModel.loading.collectAsStateWithLifecycle()
	var showSubscriptionDialog by remember { mutableStateOf(false) }

	WelcomeAccountScreen(
		loading = loading,
		onLogInClick = {
			navController.navigateAndForget(Route.Login)
		},
		onStartClick = {
			if (!loading) {
				if (viewModel.isBillingAvailable()) {
					if (activeSubscription) {
						showSubscriptionDialog = true
					} else {
						navController.replaceCurrentWith(Route.Generating)
					}
				} else {
					appUiState.managerState.accountLinks?.signUp?.let {
						Timber.d("Create url: $it")
						context.openWebUrl(it)
					}
					navController.replaceCurrentWith(Route.Login)
				}
			}
		},
	)

	ExistingSubscriptionModal(
		showSubscriptionDialog = showSubscriptionDialog,
		onClickLogin = {
			showSubscriptionDialog = false
			navController.navigateAndForget(Route.Login)
		},
		onClickCancel = {
			showSubscriptionDialog = false
		},
		onDismiss = {
			showSubscriptionDialog = false
		},
	)
}

@Composable
fun WelcomeAccountScreen(
	loading: Boolean,
	onLogInClick: () -> Unit,
	onStartClick: () -> Unit,
	onContinueWithSocialClick: () -> Unit = onStartClick,
	onLoginWithPassphraseClick: () -> Unit = onLogInClick,
) {
	Column(
		horizontalAlignment = Alignment.CenterHorizontally,
		modifier = Modifier
			.fillMaxSize()
			.background(MaterialTheme.colorScheme.background)
			.padding(horizontal = 24.dp.scaledWidth(), vertical = 24.dp.scaledHeight()),
	) {
		Image(
			painter = painterResource(R.drawable.app_label),
			contentDescription = stringResource(R.string.app_name),
			colorFilter = ColorFilter.tint(MaterialTheme.colorScheme.onBackground),
			modifier = Modifier
				.width(110.dp.scaledHeight()),
		)
		Spacer(modifier = Modifier.height(48.dp.scaledHeight()))
		Text(
			text = stringResource(R.string.get_started),
			style = MaterialTheme.typography.headlineSmall,
			color = MaterialTheme.colorScheme.onBackground,
			textAlign = TextAlign.Center,
			fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
		)

		Spacer(modifier = Modifier.height(48.dp.scaledHeight()))

		WelcomeAccountBlock(
			title = stringResource(R.string.account_welcome_create_title),
			description = stringResource(R.string.account_welcome_create_description),
			button = {
				MainStyledButton(
					onClick = onStartClick,
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
		HorizontalDivider(color = MaterialTheme.colorScheme.outline.copy(alpha = 0.35f))
		WelcomeAccountBlock(
			title = stringResource(R.string.account_welcome_social_title),
			description = stringResource(R.string.account_welcome_social_description),
			button = {
				OutlineStyledButton(
					onClick = onContinueWithSocialClick,
					content = {
						Text(
							text = stringResource(R.string.account_welcome_social_button),
							style = CustomTypography.buttonMain,
							color = MaterialTheme.colorScheme.onBackground,
						)
						Icon(
							Icons.AutoMirrored.Outlined.OpenInNew,
							stringResource(R.string.go),
							modifier = Modifier.padding(start = 4.dp).size(16.dp),
							tint = MaterialTheme.colorScheme.onBackground,
						)
					},
					modifier = Modifier
						.fillMaxWidth()
						.height(54.dp.scaledHeight()),
					borderColor = MaterialTheme.colorScheme.onBackground,
					backgroundColor = MaterialTheme.colorScheme.background,
				)
			},
		)
		HorizontalDivider(color = MaterialTheme.colorScheme.outline.copy(alpha = 0.35f))
		WelcomeAccountBlock(
			title = stringResource(R.string.account_welcome_login_title),
			button = {
				OutlineStyledButton(
					onClick = onLoginWithPassphraseClick,
					content = {
						Text(
							text = stringResource(R.string.account_welcome_login_button),
							style = CustomTypography.buttonMain,
							color = MaterialTheme.colorScheme.onBackground,
						)
					},
					modifier = Modifier
						.fillMaxWidth()
						.height(54.dp.scaledHeight()),
					borderColor = MaterialTheme.colorScheme.onBackground,
					backgroundColor = MaterialTheme.colorScheme.background,
				)
			},
		)

		Spacer(modifier = Modifier.weight(1f))

		Text(
			text = buildAnnotatedString {
				append(stringResource(R.string.account_welcome_privacy_start))
				append("\n")
				withStyle(
					style = SpanStyle(
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
					style = SpanStyle(
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
				.padding(bottom = 12.dp.scaledHeight()),
		)
	}
}

@Composable
private fun WelcomeAccountBlock(title: String, description: String? = null, button: @Composable () -> Unit) {
	Column(
		modifier = Modifier.fillMaxWidth().padding(vertical = 24.dp),
	) {
		Row(
			verticalAlignment = Alignment.CenterVertically,
			modifier = Modifier.fillMaxWidth(),
		) {
			Text(
				text = title,
				style = MaterialTheme.typography.titleMedium,
				color = MaterialTheme.colorScheme.onBackground,
				fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
			)
		}
		description?.let { text ->
			Spacer(modifier = Modifier.height(8.dp.scaledHeight()))
			Text(
				text = text,
				style = MaterialTheme.typography.bodyMedium,
				color = MaterialTheme.colorScheme.outline,
				fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
				modifier = Modifier.padding(start = 2.dp),
			)
		}
		Spacer(modifier = Modifier.height(16.dp.scaledHeight()))
		button()
	}
}

@Preview(uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
internal fun PreviewWelcomeAccountScreenRebuilt() {
	NymVPNTheme(Theme.default()) {
		WelcomeAccountScreen(
			loading = false,
			onLogInClick = {},
			onStartClick = {},
			onContinueWithSocialClick = {},
			onLoginWithPassphraseClick = {},
		)
	}
}
