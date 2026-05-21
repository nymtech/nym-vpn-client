package net.nymtech.nymvpn.ui.screens.auth

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.navigationBars
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.ModalBottomSheet
import androidx.compose.material3.rememberModalBottomSheetState
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.unit.dp

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun AuthBottomSheet(isVisible: Boolean, isMnemonicStored: Boolean, initialRoute: AuthRoute = AuthRoute.Welcome, onDismissRequest: () -> Unit, onAuthSuccess: () -> Unit, onSaveToPasswordManager: (passphrase: String) -> Unit, onWelcomeShown: () -> Unit = {}) {
	if (!isVisible || isMnemonicStored) return

	val sheetState = rememberModalBottomSheetState(skipPartiallyExpanded = true)

	val containerColor = MaterialTheme.colorScheme.surface

	ModalBottomSheet(
		onDismissRequest = onDismissRequest,
		sheetState = sheetState,
		contentWindowInsets = { WindowInsets.navigationBars },
		containerColor = containerColor,
		dragHandle = {
			Box(
				modifier = Modifier
					.fillMaxWidth()
					.background(containerColor)
					.padding(top = 8.dp),
				contentAlignment = Alignment.Center,
			) {
				Box(
					modifier = Modifier
						.size(width = 32.dp, height = 4.dp)
						.clip(RoundedCornerShape(2.dp))
						.background(MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.4f)),
				)
			}
		},
	) {
		AuthComponent(
			initialRoute = initialRoute,
			onAuthSuccess = {
				onAuthSuccess()
				onDismissRequest()
			},
			onSaveToPasswordManager = onSaveToPasswordManager,
			onWelcomeShown = onWelcomeShown,
		)
	}
}
