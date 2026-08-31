package net.nymtech.nymvpn.ui.screens.main

import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.common.snackbar.AlertAction
import net.nymtech.nymvpn.ui.common.snackbar.AlertController
import net.nymtech.nymvpn.ui.common.snackbar.AlertId
import net.nymtech.nymvpn.ui.common.snackbar.AlertMessage
import net.nymtech.nymvpn.ui.common.snackbar.AlertType
import net.nymtech.nymvpn.ui.model.ConnectionState
import net.nymtech.nymvpn.ui.screens.account.info.AutologinState
import net.nymtech.nymvpn.ui.screens.settings.components.ExpiryState
import net.nymtech.nymvpn.util.extensions.toUserMessage
import nym_vpn_lib_types.AccountControllerState

@Composable
fun MainAlerts(
	connectionState: ConnectionState,
	accountState: AccountControllerState,
	autologinState: AutologinState,
	expiryState: ExpiryState?,
	validUntilDate: String,
	expiryBannerDismissed: Boolean,
	hasSubscriptionHistory: Boolean,
	onRetryConnect: () -> Unit,
	onDismissExpiryBanner: () -> Unit,
	onRenewSubscription: () -> Unit,
	onNavigateToSelectPlan: () -> Unit,
) {
	val context = LocalContext.current
	var expiredAlertShown by rememberSaveable { mutableStateOf(false) }

	val autologinErrorTitle = stringResource(R.string.account_info_autologin_error)
	LaunchedEffect(autologinState) {
		if (autologinState is AutologinState.Error) {
			AlertController.show(AlertMessage(type = AlertType.Negative, title = autologinErrorTitle))
		}
	}

	val expiryWarningTitle = stringResource(R.string.banner_plan_expires_text, validUntilDate)
	val expiryRenewLabel = stringResource(R.string.banner_renew_text)
	val expiredTitle = stringResource(R.string.error_expired_subscription_title)
	val expiredBody = stringResource(R.string.error_expired_subscription_description)
	val expiredAction = stringResource(R.string.error_expired_subscription_button)
	val noSubscriptionTitle = stringResource(R.string.error_no_subscription_title)
	val noSubscriptionBody = stringResource(R.string.error_no_subscription_description)
	val noSubscriptionAction = stringResource(R.string.error_no_subscription_button)
	LaunchedEffect(expiryState, expiryBannerDismissed, hasSubscriptionHistory) {
		when {
			!expiryBannerDismissed && expiryState == ExpiryState.WARNING -> AlertController.show(
				AlertMessage(
					type = AlertType.Warning,
					title = expiryWarningTitle,
					action = AlertAction(expiryRenewLabel) {
						onDismissExpiryBanner()
						onRenewSubscription()
					},
					duration = Long.MAX_VALUE,
					onDismiss = { onDismissExpiryBanner() },
					id = AlertId.ExpiryWarning,
				),
			)
			expiryState == ExpiryState.EXPIRED && !expiredAlertShown -> {
				expiredAlertShown = true
				AlertController.show(
					AlertMessage(
						type = AlertType.Error,
						title = if (hasSubscriptionHistory) expiredTitle else noSubscriptionTitle,
						body = if (hasSubscriptionHistory) expiredBody else noSubscriptionBody,
						action = AlertAction(if (hasSubscriptionHistory) expiredAction else noSubscriptionAction) { onNavigateToSelectPlan() },
						duration = Long.MAX_VALUE,
						id = AlertId.Expired,
					),
				)
			}
			else -> {
				AlertController.dismiss(id = AlertId.ExpiryWarning)
				AlertController.dismiss(id = AlertId.Expired)
			}
		}
	}

	val pendingSubscriptionTitle = stringResource(R.string.account_subscription_pending_title)
	val pendingSubscriptionBody = stringResource(R.string.account_subscription_pending_description)
	LaunchedEffect(accountState) {
		if (accountState is AccountControllerState.PendingSubscription) {
			AlertController.show(
				AlertMessage(
					type = AlertType.Neutral,
					title = pendingSubscriptionTitle,
					body = pendingSubscriptionBody,
					duration = Long.MAX_VALUE,
					id = AlertId.PendingSubscription,
				),
			)
		} else {
			AlertController.dismiss(id = AlertId.PendingSubscription)
		}
	}

	val retryLabel = stringResource(R.string.try_reconnecting)
	val connectionFailedLabel = stringResource(R.string.connection_failed)
	LaunchedEffect(connectionState) {
		when (val state = connectionState) {
			is ConnectionState.Error -> AlertController.show(
				AlertMessage(
					type = AlertType.Error,
					title = state.reason.toUserMessage(context).ifEmpty { connectionFailedLabel },
					action = AlertAction(retryLabel) { onRetryConnect() },
					duration = Long.MAX_VALUE,
					id = AlertId.ConnectionError,
				),
			)
			is ConnectionState.StartFailure -> AlertController.show(
				AlertMessage(
					type = AlertType.Error,
					title = state.exception.toUserMessage(context),
					action = AlertAction(retryLabel) { onRetryConnect() },
					duration = Long.MAX_VALUE,
					id = AlertId.ConnectionError,
				),
			)
			else -> AlertController.dismiss(id = AlertId.ConnectionError)
		}
	}
}
