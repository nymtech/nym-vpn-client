package net.nymtech.nymvpn.ui.screens.auth

import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.navigationBars
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.rememberModalBottomSheetState
import androidx.compose.runtime.Composable

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun AuthBottomSheet(isVisible: Boolean, initialRoute: AuthRoute = AuthRoute.Welcome, onDismissRequest: () -> Unit, onAuthSuccess: () -> Unit, onSaveToPasswordManager: (passphrase: String) -> Unit) {
	if (!isVisible) return

	val sheetState = rememberModalBottomSheetState(skipPartiallyExpanded = true)

	ModalBottomSheet(
		onDismissRequest = onDismissRequest,
		sheetState = sheetState,
		contentWindowInsets = { WindowInsets.navigationBars },
	) {
		AuthComponent(
			initialRoute = initialRoute,
			onAuthSuccess = {
				onAuthSuccess()
				onDismissRequest()
			},
			onSaveToPasswordManager = onSaveToPasswordManager,
		)
	}
}
