package net.nymtech.nymvpn.ui.screens.account.welcome

import android.content.res.Configuration
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.asPaddingValues
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.systemBars
import androidx.compose.foundation.layout.width
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.AccountBalanceWallet
import androidx.compose.material.icons.filled.Code
import androidx.compose.material.icons.filled.Public
import androidx.compose.material.icons.filled.VerifiedUser
import androidx.compose.material.icons.outlined.Refresh
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
import androidx.compose.ui.draw.scale
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.layout.ContentScale
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.res.vectorResource
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.Route
import net.nymtech.nymvpn.ui.common.animations.SpinningIcon
import net.nymtech.nymvpn.ui.common.buttons.MainStyledButton
import net.nymtech.nymvpn.ui.common.navigation.LocalNavController
import net.nymtech.nymvpn.ui.common.snackbar.SnackbarController
import net.nymtech.nymvpn.ui.screens.account.welcome.modal.ExistingSubscriptionModal
import net.nymtech.nymvpn.ui.theme.CustomTypography
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.ui.theme.Typography
import net.nymtech.nymvpn.util.extensions.navigateAndForget
import net.nymtech.nymvpn.util.extensions.replaceCurrentWith
import net.nymtech.nymvpn.util.extensions.scaledHeight

@Composable
fun WelcomeAccountScreen(viewModel: WelcomeAccountViewModel = hiltViewModel()) {
	val context = LocalContext.current
	val navController = LocalNavController.current
	val snackbar = SnackbarController.current

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
				if (activeSubscription) {
					showSubscriptionDialog = true
				} else {
					navController.replaceCurrentWith(Route.Generating)
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
			snackbar.showMessage(context.getString(R.string.account_subscription_info))
			navController.replaceCurrentWith(Route.Generating)
		},
		onDismiss = {
			showSubscriptionDialog = false
		},
	)
}

@Composable
fun WelcomeAccountScreen(loading: Boolean, onLogInClick: () -> Unit, onStartClick: () -> Unit) {
	Column(
		horizontalAlignment = Alignment.CenterHorizontally,
		modifier = Modifier
			.fillMaxSize()
			.background(color = MaterialTheme.colorScheme.background)
			.padding(WindowInsets.systemBars.asPaddingValues()),
	) {
		Column(
			horizontalAlignment = Alignment.CenterHorizontally,
			verticalArrangement = Arrangement.Bottom,
			modifier = Modifier
				.weight(1f),
		) {
			Box {
				Image(
					painter = painterResource(id = R.drawable.background),
					contentDescription = null,
					modifier = Modifier
						.width(660.dp)
						.scale(1.2f),
					contentScale = ContentScale.None,
				)
				Icon(
					imageVector = ImageVector.vectorResource(R.drawable.app_label),
					contentDescription = "",
					modifier = Modifier
						.width(88.dp)
						.align(Alignment.Center),
					tint = MaterialTheme.colorScheme.onBackground,
				)
			}
			Text(
				text = stringResource(R.string.welcome_to_nym),
				style = MaterialTheme.typography.headlineLarge,
				color = MaterialTheme.colorScheme.onBackground,
				modifier = Modifier.padding(top = 40.dp),
				fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
			)
			Column {
				Row(
					modifier = Modifier.padding(top = 24.dp),
					verticalAlignment = Alignment.CenterVertically,
				) {
					Icon(
						imageVector = Icons.Default.VerifiedUser,
						contentDescription = "",
						modifier = Modifier.width(15.dp),
						tint = MaterialTheme.colorScheme.primary,
					)
					Spacer(modifier = Modifier.width(6.dp))
					Text(
						text = stringResource(R.string.account_welcome_anonymous),
						style = Typography.bodyMedium,
						color = MaterialTheme.colorScheme.outline,
						fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
					)
				}
				Row(
					modifier = Modifier.padding(top = 16.dp),
					verticalAlignment = Alignment.CenterVertically,
				) {
					Icon(
						imageVector = Icons.Default.Public,
						contentDescription = "",
						modifier = Modifier.width(15.dp),
						tint = MaterialTheme.colorScheme.primary,
					)
					Spacer(modifier = Modifier.width(6.dp))
					Text(
						text = stringResource(R.string.account_welcome_countries),
						style = Typography.bodyMedium,
						color = MaterialTheme.colorScheme.outline,
						fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
					)
				}
				Row(
					modifier = Modifier.padding(top = 16.dp),
					verticalAlignment = Alignment.CenterVertically,
				) {
					Icon(
						imageVector = Icons.Default.AccountBalanceWallet,
						contentDescription = "",
						modifier = Modifier.width(15.dp),
						tint = MaterialTheme.colorScheme.primary,
					)
					Spacer(modifier = Modifier.width(6.dp))
					Text(
						text = stringResource(R.string.account_welcome_payments),
						style = Typography.bodyMedium,
						color = MaterialTheme.colorScheme.outline,
						fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
					)
				}
				Row(
					modifier = Modifier.padding(top = 16.dp),
					verticalAlignment = Alignment.CenterVertically,
				) {
					Icon(
						imageVector = Icons.Default.Code,
						contentDescription = "",
						modifier = Modifier.width(15.dp),
						tint = MaterialTheme.colorScheme.primary,
					)
					Spacer(modifier = Modifier.width(6.dp))
					Text(
						text = stringResource(R.string.account_welcome_open_source),
						style = Typography.bodyMedium,
						color = MaterialTheme.colorScheme.outline,
						fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
					)
				}
			}
		}
		Column(
			horizontalAlignment = Alignment.CenterHorizontally,
			verticalArrangement = Arrangement.spacedBy(16.dp, Alignment.Bottom),
			modifier = Modifier.padding(
				top = 44.dp,
				bottom = 24.dp,
				start = 16.dp,
				end = 16.dp,
			),
		) {
			MainStyledButton(
				onClick = {
					onStartClick()
				},
				content = {
					if (loading) {
						SpinningIcon(Icons.Outlined.Refresh, stringResource(R.string.refresh))
					} else {
						Text(
							stringResource(R.string.account_welcome_button),
							style = CustomTypography.buttonMain,
						)
					}
				},
				color = MaterialTheme.colorScheme.primary,
				modifier = Modifier
					.fillMaxWidth()
					.height(54.dp.scaledHeight()),
			)
			Row(
				modifier = Modifier
					.fillMaxWidth()
					.padding(vertical = 10.dp),
				horizontalArrangement = Arrangement.Center,
				verticalAlignment = Alignment.CenterVertically,
			) {
				Text(
					text = stringResource(R.string.account_welcome_have_account),
					style = Typography.titleMedium,
					color = MaterialTheme.colorScheme.outline,
					fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
				)
				Spacer(modifier = Modifier.width(6.dp))
				Text(
					text = stringResource(R.string.log_in),
					style = Typography.titleMedium,
					color = MaterialTheme.colorScheme.primary,
					fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
					modifier = Modifier.clickable {
						onLogInClick()
					},
				)
			}
		}
	}
}

@Composable
@Preview(uiMode = Configuration.UI_MODE_NIGHT_YES)
internal fun PreviewWelcomeAccountScreen() {
	NymVPNTheme(Theme.default()) {
		WelcomeAccountScreen(true, onStartClick = {}, onLogInClick = {})
	}
}
