package net.nymtech.nymvpn.ui.screens.scan

import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.navigationBars
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.ModalBottomSheetProperties
import androidx.compose.material3.SheetValue
import androidx.compose.material3.rememberModalBottomSheetState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.DisposableEffect
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.rememberUpdatedState
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import net.nymtech.nymvpn.ui.common.NymModalBottomSheet
import net.nymtech.nymvpn.ui.common.NymSheetDragHandle
import net.nymtech.nymvpn.ui.screens.scan.components.RedeemCodeView

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun RedeemCodeSheet(onDismiss: () -> Unit, viewModel: RedeemCodeViewModel = hiltViewModel()) {
	val uiState by viewModel.uiState.collectAsStateWithLifecycle()
	val openAcceptedCode = rememberOpenAcceptedCode()
	val loading by rememberUpdatedState(uiState.loading)
	// A redeem in flight can't be cancelled, so the sheet stays open.
	val sheetState = rememberModalBottomSheetState(
		skipPartiallyExpanded = true,
		confirmValueChange = { it != SheetValue.Hidden || !loading },
	)
	val containerColor = MaterialTheme.colorScheme.surface

	LaunchedEffect(Unit) {
		viewModel.accepted.collect(openAcceptedCode)
	}

	DisposableEffect(viewModel) {
		onDispose(viewModel::reset)
	}

	NymModalBottomSheet(
		onDismissRequest = { if (!loading) onDismiss() },
		sheetState = sheetState,
		properties = ModalBottomSheetProperties(shouldDismissOnClickOutside = !uiState.loading),
		contentWindowInsets = { WindowInsets.navigationBars },
		containerColor = containerColor,
		dragHandle = { NymSheetDragHandle(containerColor) },
	) {
		RedeemCodeView(
			uiState = uiState,
			onCodeChange = viewModel::onCodeChange,
			onSubmit = viewModel::submit,
		)
	}
}
