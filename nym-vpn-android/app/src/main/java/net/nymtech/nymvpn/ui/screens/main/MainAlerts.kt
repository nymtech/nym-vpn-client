package net.nymtech.nymvpn.ui.screens.main

import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.lifecycle.compose.LifecycleResumeEffect
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.common.snackbar.AlertAction
import net.nymtech.nymvpn.ui.common.snackbar.AlertController
import net.nymtech.nymvpn.ui.common.snackbar.AlertId
import net.nymtech.nymvpn.ui.common.snackbar.AlertMessage
import net.nymtech.nymvpn.ui.common.snackbar.AlertType
import net.nymtech.nymvpn.ui.model.ConnectionState
import net.nymtech.nymvpn.ui.screens.account.info.AutologinState
import net.nymtech.nymvpn.ui.screens.main.modal.ExpiryReminderModal
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
	isLoggedIn: Boolean,
	isScreenLockSet: () -> Boolean,
	onSetUpScreenLock: () -> Unit,
) {
	val context = LocalContext.current
	var expiredAlertShown by rememberSaveable { mutableStateOf(false) }

	val autologinErrorTitle = stringResource(R.string.account_info_autologin_error)
	LaunchedEffect(autologinState) {
		if (autologinState is AutologinState.Error) {
			AlertController.show(AlertMessage(type = AlertType.Negative, title = autologinErrorTitle))
		}
	}

	val expiryWarningTitle = stringResource(R.string.account_info_expires_text, validUntilDate)
	val expiryWarningBody = stringResource(R.string.renewal_reminder_description)
	val renewLabel = stringResource(R.string.error_expired_subscription_button)
	val expiredTitle = stringResource(R.string.error_expired_subscription_title)
	val expiredBody = stringResource(R.string.error_expired_subscription_description)
	val noSubscriptionTitle = stringResource(R.string.error_no_subscription_title)
	val noSubscriptionBody = stringResource(R.string.error_no_subscription_description)
	val noSubscriptionAction = stringResource(R.string.error_no_subscription_button)

	// Upcoming-expiry reminder, surfaced as a dialog (previously an AlertController
	// snackbar). The warning's dismissal persists via the view model's banner flag;
	// the expired reminder is shown once per session.
	ExpiryReminderModal(
		show = !expiryBannerDismissed && expiryState == ExpiryState.WARNING,
		title = expiryWarningTitle,
		body = expiryWarningBody,
		actionLabel = renewLabel,
		onDismiss = { onDismissExpiryBanner() },
		onConfirmClick = {
			onDismissExpiryBanner()
			onRenewSubscription()
		},
	)
	ExpiryReminderModal(
		show = expiryState == ExpiryState.EXPIRED && !expiredAlertShown,
		title = if (hasSubscriptionHistory) expiredTitle else noSubscriptionTitle,
		body = if (hasSubscriptionHistory) expiredBody else noSubscriptionBody,
		actionLabel = if (hasSubscriptionHistory) renewLabel else noSubscriptionAction,
		onDismiss = { expiredAlertShown = true },
		onConfirmClick = {
			expiredAlertShown = true
			onNavigateToSelectPlan()
		},
	)

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
					dismissable = true,
				),
			)
			is ConnectionState.StartFailure -> AlertController.show(
				AlertMessage(
					type = AlertType.Error,
					title = state.exception.toUserMessage(context),
					action = AlertAction(retryLabel) { onRetryConnect() },
					duration = Long.MAX_VALUE,
					id = AlertId.ConnectionError,
					dismissable = true,
				),
			)
			else -> AlertController.dismiss(id = AlertId.ConnectionError)
		}
	}

	val noScreenLockTitle = stringResource(R.string.screen_lock_card_title)
	val noScreenLockBody = stringResource(R.string.screen_lock_card_body)
	val noScreenLockAction = stringResource(R.string.screen_lock_setup_button)
	val currentIsScreenLockSet by rememberUpdatedState(isScreenLockSet)
	val currentOnSetUpScreenLock by rememberUpdatedState(onSetUpScreenLock)
	val activeAlert by AlertController.message.collectAsStateWithLifecycle()
	val isAlertSlotFree = activeAlert == null
	// session-only: the alert comes back on the next app launch
	var noScreenLockDismissed by rememberSaveable { mutableStateOf(false) }
	// the alert's action button also triggers onDismiss, so only the close icon counts as a dismissal
	var noScreenLockSetupClicked by remember { mutableStateOf(false) }
	LifecycleResumeEffect(isLoggedIn, isAlertSlotFree, noScreenLockDismissed) {
		if (isLoggedIn && !currentIsScreenLockSet() && !noScreenLockDismissed) {
			val current = AlertController.message.value
			if (current == null || current.id == AlertId.NoScreenLock) {
				AlertController.show(
					AlertMessage(
						type = AlertType.Warning,
						title = noScreenLockTitle,
						body = noScreenLockBody,
						action = AlertAction(noScreenLockAction) {
							noScreenLockSetupClicked = true
							currentOnSetUpScreenLock()
						},
						duration = Long.MAX_VALUE,
						onDismiss = {
							if (noScreenLockSetupClicked) noScreenLockSetupClicked = false else noScreenLockDismissed = true
						},
						id = AlertId.NoScreenLock,
						dismissable = true,
					),
				)
			}
		} else {
			AlertController.dismiss(id = AlertId.NoScreenLock)
		}
		onPauseOrDispose {}
	}
}
