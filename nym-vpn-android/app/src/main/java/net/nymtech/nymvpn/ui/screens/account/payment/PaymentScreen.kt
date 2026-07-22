package net.nymtech.nymvpn.ui.screens.account.payment

import android.app.Activity
import android.content.res.Configuration
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import kotlinx.coroutines.delay
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.AppUiState
import net.nymtech.nymvpn.ui.Route
import net.nymtech.nymvpn.ui.common.animations.PulsingDotsWave
import net.nymtech.nymvpn.ui.common.navigation.LocalNavController
import net.nymtech.nymvpn.ui.common.snackbar.SnackbarController
import net.nymtech.nymvpn.ui.theme.LocalNymColors
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.util.StringValue
import net.nymtech.nymvpn.util.extensions.navigateAndForget
import net.nymtech.nymvpn.util.extensions.replaceCurrentWith
import nym_vpn_lib_types.AccountControllerState
import timber.log.Timber

@Composable
fun PaymentScreen(appUiState: AppUiState, productId: String, viewModel: PaymentViewModel = hiltViewModel()) {
	val navController = LocalNavController.current
	val context = LocalContext.current
	val activity = context as? Activity
	val userId = appUiState.managerState.accountId

	var animationStart by rememberSaveable { mutableStateOf(false) }
	val accountState by viewModel.accountState.collectAsState()
	val nextRoute by viewModel.nextRoute.collectAsState()

	LaunchedEffect(activity, productId) {
		activity?.let {
			viewModel.startPurchaseFlow(it, productId, userId)
		}
	}

	LaunchedEffect(Unit) {
		viewModel.events.collect { event ->
			when (event) {
				is PaymentUiEvent.PaymentError,
				is PaymentUiEvent.UserCanceled,
				-> {
					SnackbarController.showMessage(StringValue.StringResource(R.string.account_payment_error))
					navController.replaceCurrentWith(Route.SelectPlan)
				}

				is PaymentUiEvent.PaymentSuccess -> {
					animationStart = true
					viewModel.refreshAccount()
				}

				is PaymentUiEvent.SubscriptionOwned -> {
					animationStart = true
					viewModel.refreshAccount()
				}

				PaymentUiEvent.PaymentPending -> {
					animationStart = true
				}
			}
		}
	}

	PaymentScreen(
		start = animationStart,
		accountState = accountState,
		onAnimationEnd = {
			val destination = nextRoute ?: Route.Main()
			viewModel.consumeNextRoute()
			navController.navigateAndForget(destination)
		},
	)
}

@Composable
fun PaymentScreen(accountState: AccountControllerState?, start: Boolean, onAnimationEnd: () -> Unit) {
	val backgroundColor =
		if (accountState == AccountControllerState.ReadyToConnect) {
			MaterialTheme.colorScheme.primary
		} else {
			MaterialTheme.colorScheme.surfaceContainer
		}

	var textData by remember {
		mutableStateOf(
			Pair(
				R.string.account_payment_processing,
				R.string.account_payment_verifying,
			),
		)
	}

	LaunchedEffect(start, accountState) {
		Timber.d("accountState UI: $accountState")
		if (!start || accountState == null) return@LaunchedEffect

		when (accountState) {
			is AccountControllerState.ReadyToConnect -> {
				textData = Pair(
					R.string.account_payment_welcome,
					R.string.account_payment_protected,
				)
				delay(3_000)
				onAnimationEnd()
			}

			is AccountControllerState.Syncing -> {
				textData = Pair(
					R.string.account_payment_retrieving,
					R.string.account_payment_credentials,
				)
			}

			else -> Unit
		}
	}

	Column(
		modifier = Modifier
			.fillMaxSize()
			.background(MaterialTheme.colorScheme.background),
		horizontalAlignment = Alignment.CenterHorizontally,
	) {
		Row(
			modifier = Modifier
				.padding(horizontal = 16.dp)
				.fillMaxWidth()
				.height(4.dp),
			horizontalArrangement = Arrangement.spacedBy(4.dp),
		) {
			Box(
				modifier = Modifier
					.weight(1f)
					.fillMaxHeight()
					.background(
						MaterialTheme.colorScheme.primary,
						shape = RoundedCornerShape(size = 4.dp),
					),
			)
			Box(
				modifier = Modifier
					.weight(1f)
					.fillMaxHeight()
					.background(
						backgroundColor,
						shape = RoundedCornerShape(size = 4.dp),
					),
			)
		}

		Column(
			horizontalAlignment = Alignment.CenterHorizontally,
			modifier = Modifier.padding(top = 200.dp),
		) {
			val nymColors = LocalNymColors.current
			Box(
				modifier = Modifier
					.size(56.dp)
					.background(
						color = nymColors.iconBackground,
						shape = RoundedCornerShape(size = 8.dp),
					)
					.border(
						width = 1.dp,
						color = nymColors.iconBorder,
						shape = RoundedCornerShape(size = 8.dp),
					),
			) {
				PulsingDotsWave(
					modifier = Modifier
						.align(Alignment.Center)
						.padding(8.dp),
				)
			}

			Text(
				text = stringResource(textData.first),
				style = MaterialTheme.typography.titleMedium,
				textAlign = TextAlign.Center,
				color = MaterialTheme.colorScheme.onPrimaryContainer,
				modifier = Modifier.padding(top = 16.dp, start = 40.dp, end = 40.dp),
			)

			Text(
				text = stringResource(textData.second),
				style = MaterialTheme.typography.bodyMedium,
				textAlign = TextAlign.Center,
				modifier = Modifier.padding(top = 16.dp, start = 40.dp, end = 40.dp),
				color = MaterialTheme.colorScheme.onBackground,
			)
		}
	}
}

@Preview(uiMode = Configuration.UI_MODE_NIGHT_YES, showBackground = true)
@Composable
private fun PreviewPaymentScreen() {
	NymVPNTheme(Theme.default()) {
		PaymentScreen(accountState = AccountControllerState.Syncing, start = true, onAnimationEnd = {})
	}
}
