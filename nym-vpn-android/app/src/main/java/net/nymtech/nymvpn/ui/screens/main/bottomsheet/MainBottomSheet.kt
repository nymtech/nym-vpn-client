package net.nymtech.nymvpn.ui.screens.main.bottomsheet

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
import androidx.compose.runtime.rememberUpdatedState
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.layout.onSizeChanged
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.ui.AppUiState
import net.nymtech.nymvpn.ui.screens.main.bottomsheet.auth.AuthComponent
import net.nymtech.nymvpn.ui.screens.main.bottomsheet.processing.ProcessingComponent

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun MainBottomSheet(
	content: MainBottomSheetContent,
	onDismissRequest: () -> Unit,
	onAuthSuccess: () -> Unit,
	onLoginProcessingStart: (passphrase: String) -> Unit,
	onWelcomeShown: () -> Unit = {},
	appUiState: AppUiState,
	authSheetMinHeightPx: Int = 0,
	onAuthSheetHeightChange: (Int) -> Unit = {},
) {
	if (content is MainBottomSheetContent.Hidden) return

	val isProcessing = content is MainBottomSheetContent.LoginProcessing
	val isProcessingState = rememberUpdatedState(isProcessing)
	val sheetState = rememberModalBottomSheetState(
		skipPartiallyExpanded = true,
		confirmValueChange = { newValue ->
			!isProcessingState.value || newValue != SheetValue.Hidden
		},
	)
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
						onWelcomeShown = onWelcomeShown,
						appUiState = appUiState,
					)
				}
			}
			MainBottomSheetContent.LoginProcessing -> {
				ProcessingComponent(
					onProcessingComplete = onDismissRequest,
					authSheetMinHeightPx = authSheetMinHeightPx,
				)
			}
			MainBottomSheetContent.Hidden -> Unit
		}
	}
}
