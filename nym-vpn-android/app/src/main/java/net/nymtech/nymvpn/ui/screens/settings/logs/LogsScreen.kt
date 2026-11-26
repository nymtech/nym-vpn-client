package net.nymtech.nymvpn.ui.screens.settings.logs

import android.content.res.Configuration
import android.net.Uri
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.WindowInsetsSides
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.only
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.systemBars
import androidx.compose.foundation.layout.windowInsetsPadding
import androidx.compose.foundation.lazy.LazyListState
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.selection.toggleable
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.KeyboardArrowRight
import androidx.compose.material.icons.filled.Delete
import androidx.compose.material.icons.filled.KeyboardArrowDown
import androidx.compose.material.icons.filled.Share
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.derivedStateOf
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.saveable.rememberSaveable
import androidx.compose.runtime.setValue
import androidx.compose.runtime.withFrameNanos
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import kotlinx.coroutines.flow.collectLatest
import net.nymtech.logcatutil.model.LogLevel
import net.nymtech.logcatutil.model.LogMessage
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.AppUiState
import net.nymtech.nymvpn.ui.common.buttons.MainStyledButton
import net.nymtech.nymvpn.ui.common.buttons.OutlineStyledButton
import net.nymtech.nymvpn.ui.screens.settings.logs.components.EmptyLogsPlaceholder
import net.nymtech.nymvpn.ui.screens.settings.logs.components.LogsList
import net.nymtech.nymvpn.ui.screens.settings.logs.components.LogsPagerTabs
import net.nymtech.nymvpn.ui.screens.settings.logs.modal.LogsModal
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.util.extensions.scaledHeight
import net.nymtech.nymvpn.util.extensions.scaledWidth

/** What the screen shows. The logging level itself is set on Data, privacy & logs. */
data class LogsViewState(val logsEnabled: Boolean, val busy: Boolean, val showLive: Boolean, val selectedTab: Int, val currentLogs: List<LogMessage>, val currentScrollState: LazyListState)

class LogsActions(val onShare: () -> Unit, val onSave: () -> Unit, val onDelete: () -> Unit, val onToggleLive: () -> Unit, val onSelectTab: (Int) -> Unit)

@Composable
fun LogsScreen(appUiState: AppUiState, viewModel: LogsViewModel = hiltViewModel()) {
	var selectedTab by remember { mutableIntStateOf(0) }
	var showLive by rememberSaveable { mutableStateOf(false) }
	var showDelete by remember { mutableStateOf(false) }

	val appScrollState = rememberLazyListState()
	val tunnelScrollState = rememberLazyListState()
	val libraryScrollState = rememberLazyListState()

	val context = LocalContext.current

	val appLogs by viewModel.appLogs.collectAsState()
	val tunnelLogs by viewModel.tunnelLogs.collectAsState()
	val libraryLogs by viewModel.libraryLogs.collectAsState()
	val busy by viewModel.busy.collectAsState()

	val currentLogs = when (selectedTab) {
		0 -> appLogs
		1 -> tunnelLogs
		else -> libraryLogs
	}

	val currentScrollState = when (selectedTab) {
		0 -> appScrollState
		1 -> tunnelScrollState
		else -> libraryScrollState
	}

	val createDocumentLauncher = rememberLauncherForActivityResult(
		contract = ActivityResultContracts.CreateDocument("application/zip"),
	) { uri: Uri? ->
		if (uri != null) {
			viewModel.saveLogsToUri(context, uri)
		}
	}

	LaunchedEffect(Unit) {
		viewModel.requestSaveUri.collectLatest { suggestedFileName ->
			createDocumentLauncher.launch(suggestedFileName)
		}
	}

	LogsScreenContent(
		state = LogsViewState(
			logsEnabled = appUiState.settings.logsEnabled,
			busy = busy,
			showLive = showLive,
			selectedTab = selectedTab,
			currentLogs = currentLogs,
			currentScrollState = currentScrollState,
		),
		actions = LogsActions(
			onShare = { viewModel.shareLogs(context) },
			onSave = { viewModel.downloadLogs(context) },
			onDelete = { showDelete = true },
			onToggleLive = { showLive = !showLive },
			onSelectTab = { selectedTab = it },
		),
	)

	// Deleting can't be undone, so it keeps its confirm step.
	LogsModal(
		show = showDelete,
		onDismiss = { showDelete = false },
		onConfirm = {
			viewModel.deleteLogs()
			showDelete = false
		},
		title = stringResource(R.string.logs_delete_title),
		description = stringResource(R.string.logs_delete_description),
		buttonText = stringResource(R.string.logs_delete_button),
		icon = Icons.Filled.Delete,
	)
}

@Composable
fun LogsScreenContent(state: LogsViewState, actions: LogsActions, modifier: Modifier = Modifier) {
	var didInitialScroll by remember(state.currentScrollState) { mutableStateOf(false) }
	val viewerOpen = state.logsEnabled && state.showLive

	val isPinnedToBottom = remember(state.currentScrollState) {
		derivedStateOf {
			val total = state.currentScrollState.layoutInfo.totalItemsCount
			if (total == 0) return@derivedStateOf true // treat empty as pinned
			val lastVisible = state.currentScrollState.layoutInfo.visibleItemsInfo.lastOrNull()?.index
			lastVisible == total - 1
		}
	}

	LaunchedEffect(viewerOpen, state.selectedTab) {
		didInitialScroll = false
	}

	LaunchedEffect(state.currentLogs.size, viewerOpen, state.selectedTab) {
		if (!viewerOpen || didInitialScroll || state.currentLogs.isEmpty()) return@LaunchedEffect

		withFrameNanos { }

		state.currentScrollState.scrollToItem(state.currentLogs.lastIndex)
		didInitialScroll = true
	}

	LaunchedEffect(state.currentLogs.size) {
		if (!viewerOpen || !didInitialScroll) return@LaunchedEffect

		if (isPinnedToBottom.value && state.currentLogs.isNotEmpty()) {
			state.currentScrollState.scrollToItem(state.currentLogs.lastIndex)
		}
	}

	Column(
		verticalArrangement = Arrangement.spacedBy(16.dp),
		modifier = modifier
			.fillMaxSize()
			.imePadding()
			.padding(horizontal = 24.dp.scaledWidth())
			.windowInsetsPadding(WindowInsets.systemBars.only(WindowInsetsSides.Bottom)),
	) {
		if (!state.logsEnabled) {
			Text(
				text = stringResource(R.string.logs_off_hint),
				style = MaterialTheme.typography.bodyMedium,
				color = MaterialTheme.colorScheme.onBackground,
				modifier = Modifier.padding(top = 24.dp),
			)
			// Files from before logging was turned off stay on the device, so they can still be deleted.
			DeleteLogsButton(onClick = actions.onDelete, modifier = Modifier.fillMaxWidth())
			return@Column
		}

		Column(
			verticalArrangement = Arrangement.spacedBy(8.dp),
			modifier = Modifier.padding(top = 16.dp),
		) {
			MainStyledButton(
				onClick = actions.onShare,
				enabled = !state.busy,
				content = {
					Row(
						verticalAlignment = Alignment.CenterVertically,
						horizontalArrangement = Arrangement.spacedBy(8.dp),
					) {
						Icon(Icons.Filled.Share, contentDescription = null, modifier = Modifier.size(20.dp))
						Text(
							text = stringResource(R.string.logs_share_logs),
							style = MaterialTheme.typography.titleMedium,
						)
					}
				},
				modifier = Modifier
					.fillMaxWidth()
					.height(48.dp.scaledHeight()),
				shape = RoundedCornerShape(12.dp),
			)
			Text(
				text = stringResource(R.string.logs_share_description),
				style = MaterialTheme.typography.bodySmall,
				color = MaterialTheme.colorScheme.onBackground,
			)
		}

		// Save writes to shared storage, so it carries its own privacy line, as Share does.
		Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
			Row(
				horizontalArrangement = Arrangement.spacedBy(8.dp),
				modifier = Modifier.fillMaxWidth(),
			) {
				OutlineStyledButton(
					onClick = actions.onSave,
					enabled = !state.busy,
					content = {
						Text(
							text = stringResource(R.string.logs_save_logs),
							style = MaterialTheme.typography.bodyMedium,
							color = MaterialTheme.colorScheme.onPrimaryContainer,
						)
					},
					borderColor = MaterialTheme.colorScheme.onPrimaryContainer,
					modifier = Modifier
						.weight(1f)
						.height(44.dp.scaledHeight()),
					shape = RoundedCornerShape(12.dp),
				)
				DeleteLogsButton(onClick = actions.onDelete, modifier = Modifier.weight(1f))
			}
			Text(
				text = stringResource(R.string.logs_download_description),
				style = MaterialTheme.typography.bodySmall,
				color = MaterialTheme.colorScheme.onBackground,
			)
		}

		Card(
			modifier = if (state.showLive) Modifier.fillMaxWidth().weight(1f) else Modifier.fillMaxWidth(),
			shape = RoundedCornerShape(8.dp),
			colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.primaryContainer),
		) {
			Row(
				horizontalArrangement = Arrangement.SpaceBetween,
				verticalAlignment = Alignment.CenterVertically,
				modifier = Modifier
					.fillMaxWidth()
					.toggleable(value = state.showLive, role = Role.Switch, onValueChange = { actions.onToggleLive() })
					.padding(16.dp),
			) {
				Text(
					text = stringResource(R.string.logs_show_live),
					style = MaterialTheme.typography.titleMedium,
					color = MaterialTheme.colorScheme.onPrimaryContainer,
				)
				Icon(
					imageVector = if (state.showLive) Icons.Filled.KeyboardArrowDown else Icons.AutoMirrored.Filled.KeyboardArrowRight,
					contentDescription = null,
					tint = MaterialTheme.colorScheme.onBackground,
				)
			}

			if (state.showLive) {
				LogsPagerTabs(
					selectedTab = state.selectedTab,
					onSelectTab = actions.onSelectTab,
				)

				Box(
					modifier = Modifier
						.padding(8.dp)
						.fillMaxWidth(),
				) {
					if (state.currentLogs.isEmpty()) {
						EmptyLogsPlaceholder()
					} else {
						LogsList(
							logs = state.currentLogs,
							lazyColumnListState = state.currentScrollState,
							modifier = Modifier.fillMaxSize().background(MaterialTheme.colorScheme.primaryContainer),
						)
					}
				}
			}
		}
	}
}

@Composable
private fun DeleteLogsButton(onClick: () -> Unit, modifier: Modifier = Modifier) {
	OutlineStyledButton(
		onClick = onClick,
		content = {
			Text(
				text = stringResource(R.string.logs_delete_button),
				style = MaterialTheme.typography.bodyMedium,
				color = MaterialTheme.colorScheme.error,
			)
		},
		borderColor = MaterialTheme.colorScheme.error,
		modifier = modifier.height(44.dp.scaledHeight()),
		shape = RoundedCornerShape(12.dp),
	)
}

@Preview(uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
internal fun PreviewLogsScreenContent() {
	NymVPNTheme(Theme.default()) {
		val mockLogs = listOf(
			LogMessage(
				time = "2026-01-22 11:01:53.412",
				pid = "5693",
				tid = "5693",
				level = LogLevel.INFO,
				tag = "app",
				message = "AppCreate build=debug",
				epochMillis = 1L,
			),
			LogMessage(
				time = "2026-01-22 11:01:59.399",
				pid = "5693",
				tid = "5802",
				level = LogLevel.DEBUG,
				tag = "libnymvpn",
				message = "hickory_proto::xfer::dns_multiplexer: request timed out: 32446",
				epochMillis = 1L,
			),
		)

		Box(
			modifier = Modifier
				.fillMaxSize()
				.background(MaterialTheme.colorScheme.background),
		) {
			LogsScreenContent(
				state = LogsViewState(
					logsEnabled = true,
					busy = false,
					showLive = true,
					selectedTab = 0,
					currentLogs = mockLogs,
					currentScrollState = rememberLazyListState(),
				),
				actions = LogsActions(onShare = {}, onSave = {}, onDelete = {}, onToggleLive = {}, onSelectTab = {}),
			)
		}
	}
}
