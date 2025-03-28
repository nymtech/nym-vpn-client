package net.nymtech.nymvpn.ui.screens.settings.logs

import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.material3.Scaffold
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import net.nymtech.nymvpn.ui.screens.settings.logs.components.AutoScrollEffect
import net.nymtech.nymvpn.ui.screens.settings.logs.components.DeleteLogsModal
import net.nymtech.nymvpn.ui.screens.settings.logs.components.LogsBottomBar
import net.nymtech.nymvpn.ui.screens.settings.logs.components.LogsList
import net.nymtech.nymvpn.ui.screens.settings.logs.components.ScrollToBottomFab

@Composable
fun LogsScreen(viewModel: LogsViewModel = hiltViewModel()) {
	val lazyColumnListState = rememberLazyListState()
	var isAutoScrolling by remember { mutableStateOf(true) }
	var showModal by remember { mutableStateOf(false) }
	var lastScrollPosition by remember { mutableIntStateOf(0) }

	val context = LocalContext.current

	val logs = viewModel.logs

	AutoScrollEffect(
		logsSize = logs.size,
		lazyColumnListState = lazyColumnListState,
		isAutoScrolling = isAutoScrolling,
		onAutoScrollingChange = { isAutoScrolling = it },
		lastScrollPosition = lastScrollPosition,
		onLastScrollPositionChange = { lastScrollPosition = it },
	)

	Scaffold(
		floatingActionButton = {
			ScrollToBottomFab(onClick = { isAutoScrolling = true })
		},
		contentWindowInsets = WindowInsets(0.dp),
		bottomBar = {
			LogsBottomBar(
				onShareClick = { viewModel.shareLogs(context) },
				onDeleteClick = { showModal = true },
			)
		},
	) { paddingValues ->
		LogsList(
			logs = logs,
			lazyColumnListState = lazyColumnListState,
			modifier = Modifier.padding(paddingValues),
		)
	}

	DeleteLogsModal(
		show = showModal,
		onDismiss = { showModal = false },
		onConfirm = {
			viewModel.deleteLogs()
			showModal = false
		},
	)
}
