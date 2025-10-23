package net.nymtech.nymvpn.ui.screens.settings.logs

import android.net.Uri
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.material3.Scaffold
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import kotlinx.coroutines.flow.collectLatest
import net.nymtech.nymvpn.ui.screens.settings.logs.components.AutoScrollEffect
import net.nymtech.nymvpn.ui.screens.settings.logs.components.DeleteLogsModal
import net.nymtech.nymvpn.ui.screens.settings.logs.components.LogsBottomBar
import net.nymtech.nymvpn.ui.screens.settings.logs.components.LogsList
import net.nymtech.nymvpn.ui.screens.settings.logs.components.LogsTabBar
import net.nymtech.nymvpn.ui.screens.settings.logs.components.ScrollToBottomFab

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

	val createDocumentLauncher = rememberLauncherForActivityResult(
		contract = ActivityResultContracts.CreateDocument("application/zip"),
	) { uri: Uri? ->
		if (uri != null) {
			viewModel.saveLogsToUri(context, uri)
		}
	}

	AutoScrollEffect(
		logsSize = currentLogs.size,
		lazyColumnListState = currentScrollState,
		isAutoScrolling = isAutoScrolling,
		onAutoScrollingChange = { isAutoScrolling = it },
		lastScrollPosition = lastScrollPosition,
		onLastScrollPositionChange = { lastScrollPosition = it },
	)

	LaunchedEffect(Unit) {
		viewModel.requestSaveUri.collectLatest { suggestedFileName ->
			createDocumentLauncher.launch(suggestedFileName)
		}
	}

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
				LogsBottomBar { event ->
					when (event) {
						LogsBottomBarEvent.Share -> viewModel.shareLogs(context)
						LogsBottomBarEvent.Download -> viewModel.downloadLogs(context)
						LogsBottomBarEvent.Delete -> showModal = true
					}
				}
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
