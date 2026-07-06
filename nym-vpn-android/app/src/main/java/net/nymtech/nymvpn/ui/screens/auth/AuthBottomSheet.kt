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
import androidx.compose.material3.ModalBottomSheetProperties
import androidx.compose.material3.SheetValue
import androidx.compose.material3.rememberModalBottomSheetState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.key
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.layout.onSizeChanged
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.ui.AppUiState
import net.nymtech.nymvpn.ui.screens.account.login.LoginProcessingDrawer

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun AuthBottomSheet(
	content: MainBottomSheetContent,
	onDismissRequest: () -> Unit,
	onAuthSuccess: () -> Unit,
	onLoginProcessingStart: (passphrase: String) -> Unit,
	onSaveToPasswordManager: (passphrase: String) -> Unit,
	onWelcomeShown: () -> Unit = {},
	appUiState: AppUiState,
	authSheetMinHeightPx: Int = 0,
	onAuthSheetHeightChange: (Int) -> Unit = {},
) {
	if (content is MainBottomSheetContent.Hidden) return

	val isProcessing = content is MainBottomSheetContent.LoginProcessing
	val sheetState = key(isProcessing) {
		rememberModalBottomSheetState(
			skipPartiallyExpanded = true,
			confirmValueChange = { newValue ->
				!isProcessing || newValue != SheetValue.Hidden
			},
		)
	}
	val containerColor = MaterialTheme.colorScheme.surface

	ModalBottomSheet(
		onDismissRequest = {
			if (!isProcessing) onDismissRequest()
		},
		sheetState = sheetState,
		properties = ModalBottomSheetProperties(shouldDismissOnClickOutside = !isProcessing),
		contentWindowInsets = { WindowInsets.navigationBars },
		containerColor = containerColor,
		dragHandle = if (isProcessing) {
			null
		} else {
			{
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
			}
		},
	) {
		when (content) {
			is MainBottomSheetContent.Auth -> {
				Box(
					modifier = Modifier.onSizeChanged { size ->
						if (size.height > 0) onAuthSheetHeightChange(size.height)
					},
				) {
					AuthComponent(
						initialRoute = content.route,
						onAuthSuccess = {
							onAuthSuccess()
							onDismissRequest()
						},
						onLoginProcessingStart = { phrase ->
							onLoginProcessingStart(phrase)
						},
						onSaveToPasswordManager = onSaveToPasswordManager,
						onWelcomeShown = onWelcomeShown,
						appUiState = appUiState,
					)
				}
			}
			MainBottomSheetContent.LoginProcessing -> {
				LoginProcessingDrawer(
					onProcessingComplete = onDismissRequest,
					authSheetMinHeightPx = authSheetMinHeightPx,
				)
			}
			MainBottomSheetContent.Hidden -> Unit
		}
	}
}
