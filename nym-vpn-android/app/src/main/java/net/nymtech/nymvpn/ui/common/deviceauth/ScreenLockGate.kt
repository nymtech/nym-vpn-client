package net.nymtech.nymvpn.ui.common.deviceauth

import androidx.annotation.StringRes
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.hilt.navigation.compose.hiltViewModel
import timber.log.Timber

fun interface ScreenLockGate {
	fun isOpen(): Boolean
}

@Composable
fun rememberScreenLockGate(@StringRes setupBody: Int, viewModel: DeviceAuthViewModel = hiltViewModel(), onLockSetAfterSetup: () -> Unit = {}): ScreenLockGate {
	val currentOnLockSetAfterSetup by rememberUpdatedState(onLockSetAfterSetup)
	var showSetup by rememberSaveable { mutableStateOf(false) }

	ScreenLockSetupPrompt(
		setupBody = setupBody.takeIf { showSetup },
		viewModel = viewModel,
		onSetUpClick = { showSetup = false },
		onDismiss = { showSetup = false },
		onReturnFromSettings = {
			if (viewModel.isScreenLockSet()) currentOnLockSetAfterSetup()
		},
	)

	return ScreenLockGate {
		if (viewModel.isScreenLockSet()) {
			true
		} else {
			Timber.i("ScreenLockGate: blocked, no screen lock")
			showSetup = true
			false
		}
	}
}
