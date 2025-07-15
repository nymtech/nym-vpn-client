package net.nymtech.nymvpn.ui.screens.settings.logs

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.material3.Scaffold
import androidx.compose.runtime.*
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import net.nymtech.nymvpn.ui.screens.settings.logs.components.*

@Composable
fun LogsScreen(viewModel: LogsViewModel = hiltViewModel()) {
	var selectedTab by remember { mutableIntStateOf(0) }

	val nativeScrollState = rememberLazyListState()
	val vpnScrollState = rememberLazyListState()

	var isAutoScrolling by remember { mutableStateOf(true) }
	var lastScrollPosition by remember { mutableIntStateOf(0) }
	var showModal by remember { mutableStateOf(false) }

	val context = LocalContext.current

	val nativeLogs by viewModel.nativeLogs.collectAsState()
	val vpnLogs by viewModel.vpnLogs.collectAsState()

	val currentLogs = if (selectedTab == 0) nativeLogs else vpnLogs
	val currentScrollState = if (selectedTab == 0) nativeScrollState else vpnScrollState

	AutoScrollEffect(
		logsSize = currentLogs.size,
		lazyColumnListState = currentScrollState,
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
			Column {
				LogsTabBar(
					selectedTab = selectedTab,
					onSelectTab = { selectedTab = it },
				)
				LogsBottomBar(
					onShareClick = { viewModel.shareLogs(context) },
					onDeleteClick = { showModal = true },
				)
			}
		},
	) { paddingValues ->
		LogsList(
			logs = currentLogs,
			lazyColumnListState = currentScrollState,
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
