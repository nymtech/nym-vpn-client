package net.nymtech.nymvpn.ui.common.deviceauth

import androidx.annotation.StringRes
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.ui.platform.LocalContext
import androidx.fragment.app.FragmentActivity
import androidx.hilt.navigation.compose.hiltViewModel
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.manager.deviceauth.model.DeviceAuthResult
import net.nymtech.nymvpn.ui.common.snackbar.SnackbarController
import net.nymtech.nymvpn.util.StringValue
import net.nymtech.nymvpn.util.extensions.findActivity
import timber.log.Timber

enum class DeviceAuthAction(@StringRes val authTitle: Int, @StringRes val authSubtitle: Int, @StringRes val setupBody: Int) {
	REVEAL_PASSPHRASE(R.string.passphrase_title, R.string.passphrase_description, R.string.screen_lock_setup_passphrase_body),
	MANAGE_SUBSCRIPTION(R.string.account_info_manage_button, R.string.account_info_manage_auth_subtitle, R.string.screen_lock_setup_body),
	RENEW_SUBSCRIPTION(R.string.account_info_renew_auth_title, R.string.account_info_manage_auth_subtitle, R.string.screen_lock_setup_body),
	LINK_SOCIAL(R.string.account_info_add_social_action, R.string.account_info_link_auth_subtitle, R.string.screen_lock_setup_body),
}

fun interface DeviceAuth {
	fun request(action: DeviceAuthAction)
}

@Composable
fun rememberDeviceAuth(viewModel: DeviceAuthViewModel = hiltViewModel(), onAuthenticated: (DeviceAuthAction) -> Unit): DeviceAuth {
	val context = LocalContext.current
	val scope = rememberCoroutineScope()
	val currentOnAuthenticated by rememberUpdatedState(onAuthenticated)

	var setupBody by rememberSaveable { mutableStateOf<Int?>(null) }
	var pendingAction by rememberSaveable { mutableStateOf<DeviceAuthAction?>(null) }
	var inProgress by remember { mutableStateOf(false) }

	fun showSetup(action: DeviceAuthAction) {
		pendingAction = action
		setupBody = action.setupBody
	}

	fun request(action: DeviceAuthAction) {
		if (inProgress) return
		val activity = context.findActivity() as? FragmentActivity
		if (activity == null) {
			Timber.w("DeviceAuth: no FragmentActivity, can't authenticate")
			return
		}
		if (!viewModel.isScreenLockSet()) {
			showSetup(action)
			return
		}

		inProgress = true
		scope.launch {
			try {
				when (viewModel.authenticate(activity, context.getString(action.authTitle), context.getString(action.authSubtitle))) {
					DeviceAuthResult.Success -> currentOnAuthenticated(action)
					DeviceAuthResult.NotSetUp -> showSetup(action)
					DeviceAuthResult.Cancelled -> Unit
					is DeviceAuthResult.Unavailable, is DeviceAuthResult.Error ->
						SnackbarController.showMessage(StringValue.StringResource(R.string.device_auth_error))
				}
			} finally {
				inProgress = false
			}
		}
	}

	ScreenLockSetupPrompt(
		setupBody = setupBody,
		viewModel = viewModel,
		onSetUpClick = { setupBody = null },
		onDismiss = {
			setupBody = null
			pendingAction = null
		},
		onReturnFromSettings = {
			val action = pendingAction
			pendingAction = null
			if (action != null && viewModel.isScreenLockSet()) request(action)
		},
	)

	return DeviceAuth { request(it) }
}
