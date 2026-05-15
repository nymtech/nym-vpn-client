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
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.only
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.systemBars
import androidx.compose.foundation.layout.windowInsetsPadding
import androidx.compose.foundation.lazy.LazyListState
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Delete
import androidx.compose.material.icons.filled.Download
import androidx.compose.material.icons.filled.Share
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
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
import androidx.compose.runtime.setValue
import androidx.compose.runtime.withFrameNanos
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import kotlinx.coroutines.flow.collectLatest
import net.nymtech.logcatutil.model.LogLevel
import net.nymtech.logcatutil.model.LogMessage
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.AppUiState
import net.nymtech.nymvpn.ui.common.buttons.ScaledSwitch
import net.nymtech.nymvpn.ui.common.navigation.NavBarEvent
import net.nymtech.nymvpn.ui.screens.settings.logs.components.EmptyLogsPlaceholder
import net.nymtech.nymvpn.ui.screens.settings.logs.components.LogsList
import net.nymtech.nymvpn.ui.screens.settings.logs.components.LogsPagerTabs
import net.nymtech.nymvpn.ui.screens.settings.logs.modal.LogsModal
import net.nymtech.nymvpn.ui.theme.LocalNymColors
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.util.extensions.scaledWidth

@Composable
fun LogsScreen(appUiState: AppUiState, navBarEvent: NavBarEvent?, onNavBarEventConsume: () -> Unit, viewModel: LogsViewModel = hiltViewModel()) {
	var selectedTab by remember { mutableIntStateOf(0) }

	val appScrollState = rememberLazyListState()
	val tunnelScrollState = rememberLazyListState()
	val libraryScrollState = rememberLazyListState()

	var showDelete by remember { mutableStateOf(false) }
	var showShare by remember { mutableStateOf(false) }
	var showDownload by remember { mutableStateOf(false) }

	val context = LocalContext.current

	val appLogs by viewModel.appLogs.collectAsState()
	val tunnelLogs by viewModel.tunnelLogs.collectAsState()
	val libraryLogs by viewModel.libraryLogs.collectAsState()

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

	LaunchedEffect(navBarEvent) {
		when (navBarEvent) {
			NavBarEvent.LogsDeleteClicked -> {
				showDelete = true
				onNavBarEventConsume()
			}
			NavBarEvent.LogsShareClicked -> {
				showShare = true
				onNavBarEventConsume()
			}
			NavBarEvent.LogsDownloadClicked -> {
				showDownload = true
				onNavBarEventConsume()
			}
			else -> {}
		}
	}

	LaunchedEffect(Unit) {
		viewModel.requestSaveUri.collectLatest { suggestedFileName ->
			createDocumentLauncher.launch(suggestedFileName)
		}
	}

	LogsScreenContent(
		logsEnabled = appUiState.settings.logsEnabled,
		logsDebugEnabled = appUiState.vpnConfig.debugLog,
		onLogsEnable = { viewModel.onLogsEnabled(it) },
		onLogsDebugEnable = { viewModel.onLogsDebugEnabled(it) },
		selectedTab = selectedTab,
		onSelectTab = { selectedTab = it },
		currentLogs = currentLogs,
		currentScrollState = currentScrollState,
	)

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

	LogsModal(
		show = showShare,
		onDismiss = { showShare = false },
		onConfirm = {
			viewModel.shareLogs(context)
			showShare = false
		},
		title = stringResource(R.string.logs_share_title),
		description = stringResource(R.string.logs_share_description),
		buttonText = stringResource(R.string.logs_share_button),
		icon = Icons.Filled.Share,
	)

	LogsModal(
		show = showDownload,
		onDismiss = { showDownload = false },
		onConfirm = {
			viewModel.downloadLogs(context)
			showDownload = false
		},
		title = stringResource(R.string.logs_download_title),
		description = stringResource(R.string.logs_download_description),
		buttonText = stringResource(R.string.logs_download_button),
		icon = Icons.Filled.Download,
	)
}

@Composable
fun LogsScreenContent(
	logsEnabled: Boolean,
	logsDebugEnabled: Boolean,
	onLogsEnable: (Boolean) -> Unit,
	onLogsDebugEnable: (Boolean) -> Unit,
	selectedTab: Int,
	onSelectTab: (Int) -> Unit,
	currentLogs: List<LogMessage>,
	currentScrollState: LazyListState,
	modifier: Modifier = Modifier,
) {
	var didInitialScroll by remember(currentScrollState) { mutableStateOf(false) }

	val isPinnedToBottom = remember(currentScrollState) {
		derivedStateOf {
			val total = currentScrollState.layoutInfo.totalItemsCount
			if (total == 0) return@derivedStateOf true // treat empty as pinned
			val lastVisible = currentScrollState.layoutInfo.visibleItemsInfo.lastOrNull()?.index
			lastVisible == total - 1
		}
	}

	LaunchedEffect(logsEnabled, selectedTab) {
		didInitialScroll = false
	}

	LaunchedEffect(currentLogs.size, logsEnabled, selectedTab) {
		if (!logsEnabled) return@LaunchedEffect
		if (didInitialScroll) return@LaunchedEffect
		if (currentLogs.isEmpty()) return@LaunchedEffect

		withFrameNanos { }

		val lastIndex = currentLogs.lastIndex
		currentScrollState.scrollToItem(lastIndex)
		didInitialScroll = true
	}

	LaunchedEffect(currentLogs.size) {
		if (!logsEnabled) return@LaunchedEffect
		if (!didInitialScroll) return@LaunchedEffect

		if (isPinnedToBottom.value && currentLogs.isNotEmpty()) {
			currentScrollState.scrollToItem(currentLogs.lastIndex)
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
		Card(
			modifier = Modifier.fillMaxWidth()
				.padding(top = 16.dp),
			shape = RoundedCornerShape(8.dp),
			colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.primaryContainer),
		) {
			Column(
				modifier = Modifier
					.fillMaxWidth()
					.padding(horizontal = 16.dp, vertical = 16.dp),
			) {
				Row(
					modifier = Modifier.fillMaxWidth(),
					horizontalArrangement = Arrangement.SpaceBetween,
					verticalAlignment = Alignment.CenterVertically,
				) {
					Text(
						text = stringResource(R.string.logs_enable_title),
						style = MaterialTheme.typography.titleMedium,
						color = MaterialTheme.colorScheme.onPrimaryContainer,
					)
					ScaledSwitch(
						checked = logsEnabled,
						onClick = { onLogsEnable(it) },
					)
				}

				Text(
					text = stringResource(R.string.logs_enable_description),
					style = MaterialTheme.typography.bodySmall,
					color = MaterialTheme.colorScheme.onBackground,
					modifier = Modifier
						.fillMaxWidth()
						.padding(top = 8.dp),
					textAlign = TextAlign.Justify,
				)
			}
		}

		if (!logsEnabled) return@Column

		Card(
			modifier = Modifier.fillMaxWidth(),
			shape = RoundedCornerShape(8.dp),
			colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.primaryContainer),
		) {
			Column(
				modifier = Modifier
					.fillMaxWidth()
					.padding(horizontal = 16.dp, vertical = 16.dp),
			) {
				Row(
					modifier = Modifier.fillMaxWidth(),
					horizontalArrangement = Arrangement.SpaceBetween,
					verticalAlignment = Alignment.CenterVertically,
				) {
					Text(
						text = stringResource(R.string.logs_debug_enable_title),
						style = MaterialTheme.typography.titleMedium,
						color = MaterialTheme.colorScheme.onPrimaryContainer,
					)
					ScaledSwitch(
						checked = logsDebugEnabled,
						onClick = { onLogsDebugEnable(it) },
					)
				}

				Text(
					text = stringResource(R.string.logs_debug_enable_description),
					style = MaterialTheme.typography.bodySmall,
					color = MaterialTheme.colorScheme.onBackground,
					modifier = Modifier
						.fillMaxWidth()
						.padding(top = 8.dp),
					textAlign = TextAlign.Justify,
				)

				Text(
					text = stringResource(R.string.privacy_error_reports_restart),
					style = MaterialTheme.typography.bodySmall,
					color = LocalNymColors.current.warning,
					modifier = Modifier
						.fillMaxWidth()
						.padding(top = 8.dp),
				)
			}
		}

		Card(
			modifier = Modifier.fillMaxWidth(),
			shape = RoundedCornerShape(8.dp),
			colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.primaryContainer),
		) {
			LogsPagerTabs(
				selectedTab = selectedTab,
				onSelectTab = onSelectTab,
			)

			Box(
				modifier = Modifier
					.padding(8.dp)
					.fillMaxWidth(),
			) {
				if (currentLogs.isEmpty()) {
					EmptyLogsPlaceholder()
				} else {
					LogsList(
						logs = currentLogs,
						lazyColumnListState = currentScrollState,
						modifier = Modifier.fillMaxSize().background(MaterialTheme.colorScheme.primaryContainer),
					)
				}
			}
		}
	}
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
				time = "2026-01-22 11:01:53.578",
				pid = "5693",
				tid = "5777",
				level = LogLevel.INFO,
				tag = "core-backend",
				message = "BackendInitStart env=mainnet sentry=false statistics=true logLevel=debug",
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
				logsEnabled = true,
				logsDebugEnabled = true,
				onLogsEnable = {},
				onLogsDebugEnable = {},
				selectedTab = 0,
				onSelectTab = {},
				currentLogs = mockLogs,
				currentScrollState = rememberLazyListState(),
			)
		}
	}
}
