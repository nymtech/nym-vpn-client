package net.nymtech.nymvpn.ui.screens.main

import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import kotlinx.coroutines.flow.SharedFlow
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.common.snackbar.AlertAction
import net.nymtech.nymvpn.ui.common.snackbar.AlertController
import net.nymtech.nymvpn.ui.common.snackbar.AlertMessage
import net.nymtech.nymvpn.ui.common.snackbar.AlertType
import net.nymtech.nymvpn.ui.model.ConnectionState
import net.nymtech.nymvpn.ui.screens.account.info.AutologinState
import net.nymtech.nymvpn.ui.screens.settings.components.ExpiryState
import net.nymtech.nymvpn.util.extensions.toUserMessage
import nym_vpn_lib_types.AccountControllerErrorStateReason
import nym_vpn_lib_types.AccountControllerState

@Composable
fun MainAlerts(
	connectionState: ConnectionState,
	accountState: AccountControllerState,
	autologinState: AutologinState,
	expiryState: ExpiryState?,
	validUntilDate: String,
	expiryBannerDismissed: Boolean,
	registerAccountNavigation: SharedFlow<Unit>,
	onRetryConnect: () -> Unit,
	onDismissExpiryBanner: () -> Unit,
	onRenewSubscription: () -> Unit,
	onNavigateToSelectPlan: () -> Unit,
) {
	val context = LocalContext.current

	var isShowingConnectionErrorAlert by remember { mutableStateOf(false) }
	var isShowingExpiredAlert by remember { mutableStateOf(false) }
	var isShowingInactiveAccountAlert by remember { mutableStateOf(false) }
	var expiredAlertShown by rememberSaveable { mutableStateOf(false) }
	var prevWasAccountNotActive by remember { mutableStateOf<Boolean?>(null) }

	val autologinErrorTitle = stringResource(R.string.account_info_autologin_error)
	LaunchedEffect(autologinState) {
		if (autologinState is AutologinState.Error) {
			AlertController.show(AlertMessage(type = AlertType.Negative, title = autologinErrorTitle))
		}
	}

	val expiryWarningTitle = stringResource(R.string.banner_plan_expires_text, validUntilDate)
	val expiryRenewLabel = stringResource(R.string.banner_renew_text)
	LaunchedEffect(expiryState, expiryBannerDismissed) {
		if (!expiryBannerDismissed && expiryState == ExpiryState.WARNING) {
			AlertController.show(
				AlertMessage(
					type = AlertType.Warning,
					title = expiryWarningTitle,
					action = AlertAction(expiryRenewLabel) {
						onDismissExpiryBanner()
						onRenewSubscription()
					},
					duration = Long.MAX_VALUE,
					onDismiss = { onDismissExpiryBanner() },
				),
			)
		}
	}

	val expiredTitle = stringResource(R.string.error_expired_subscription_title)
	val expiredBody = stringResource(R.string.error_expired_subscription_description)
	val expiredAction = stringResource(R.string.error_expired_subscription_button)
	LaunchedEffect(expiryState) {
		if (expiryState == ExpiryState.EXPIRED && !expiredAlertShown) {
			expiredAlertShown = true
			isShowingExpiredAlert = true
			AlertController.show(
				AlertMessage(
					type = AlertType.Error,
					title = expiredTitle,
					body = expiredBody,
					action = AlertAction(expiredAction) { onNavigateToSelectPlan() },
					duration = Long.MAX_VALUE,
					onDismiss = { isShowingExpiredAlert = false },
				),
			)
		} else if (isShowingExpiredAlert && expiryState != ExpiryState.EXPIRED) {
			AlertController.dismiss()
			isShowingExpiredAlert = false
		}
	}

	val inactiveAccountTitle = stringResource(R.string.error_inactive_account)
	LaunchedEffect(accountState) {
		val isInactive = accountState is AccountControllerState.Error &&
			accountState.v1 is AccountControllerErrorStateReason.AccountStatusNotActive
		if (isInactive && prevWasAccountNotActive != true) {
			isShowingInactiveAccountAlert = true
			AlertController.show(
				AlertMessage(
					type = AlertType.Error,
					title = inactiveAccountTitle,
					duration = Long.MAX_VALUE,
					onDismiss = { isShowingInactiveAccountAlert = false },
				),
			)
		} else if (!isInactive && isShowingInactiveAccountAlert) {
			AlertController.dismiss()
			isShowingInactiveAccountAlert = false
		}
		prevWasAccountNotActive = isInactive
	}

	LaunchedEffect(Unit) {
		registerAccountNavigation.collect { onNavigateToSelectPlan() }
	}

	val retryLabel = stringResource(R.string.try_reconnecting)
	val connectionFailedLabel = stringResource(R.string.connection_failed)
	LaunchedEffect(connectionState) {
		when (val state = connectionState) {
			is ConnectionState.Error -> {
				isShowingConnectionErrorAlert = true
				AlertController.show(
					AlertMessage(
						type = AlertType.Error,
						title = state.reason.toUserMessage(context).ifEmpty { connectionFailedLabel },
						action = AlertAction(retryLabel) { onRetryConnect() },
						duration = Long.MAX_VALUE,
						onDismiss = { isShowingConnectionErrorAlert = false },
					),
				)
			}
			is ConnectionState.StartFailure -> {
				isShowingConnectionErrorAlert = true
				AlertController.show(
					AlertMessage(
						type = AlertType.Error,
						title = state.exception.toUserMessage(context),
						action = AlertAction(retryLabel) { onRetryConnect() },
						duration = Long.MAX_VALUE,
						onDismiss = { isShowingConnectionErrorAlert = false },
					),
				)
			}
			else -> if (isShowingConnectionErrorAlert) AlertController.dismiss()
		}
	}
}
