package net.nymtech.nymvpn.ui.common.deviceauth

import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.annotation.StringRes
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.ui.res.stringResource
import net.nymtech.nymvpn.ui.common.modal.ScreenLockSetupDialog
import net.nymtech.nymvpn.util.extensions.launchScreenLockSetup

@Composable
internal fun ScreenLockSetupPrompt(@StringRes setupBody: Int?, viewModel: DeviceAuthViewModel, onSetUpClick: () -> Unit, onDismiss: () -> Unit, onReturnFromSettings: () -> Unit) {
	val currentOnReturnFromSettings by rememberUpdatedState(onReturnFromSettings)
	val launcher = rememberLauncherForActivityResult(ActivityResultContracts.StartActivityForResult()) {
		currentOnReturnFromSettings()
	}

	if (setupBody == null) return
	ScreenLockSetupDialog(
		show = true,
		body = stringResource(setupBody),
		onSetUpClick = {
			onSetUpClick()
			launcher.launchScreenLockSetup(viewModel.createScreenLockSetupIntent())
		},
		onDismiss = onDismiss,
	)
}
