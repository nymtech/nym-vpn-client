package net.nymtech.nymvpn.ui.screens.settings.tunneling

import android.widget.Toast
import androidx.activity.compose.BackHandler
import androidx.compose.foundation.background
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.navigationBars
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.windowInsetsPadding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.PreviewLightDark
import androidx.compose.ui.unit.dp
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import kotlinx.coroutines.flow.collectLatest
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.common.buttons.MainStyledButton
import net.nymtech.nymvpn.ui.common.events.UiEvent
import net.nymtech.nymvpn.ui.common.navigation.LocalNavController
import net.nymtech.nymvpn.ui.common.navigation.NavBarEvent
import net.nymtech.nymvpn.ui.screens.settings.dns.modal.SaveChangesModal
import net.nymtech.nymvpn.ui.screens.settings.geoexclusion.components.WarningCard
import net.nymtech.nymvpn.ui.screens.settings.tunneling.components.AppInfoRow
import net.nymtech.nymvpn.ui.screens.settings.tunneling.components.LoadingDialog
import net.nymtech.nymvpn.ui.screens.settings.tunneling.components.SplitTunnelingInfoModal
import net.nymtech.nymvpn.ui.screens.settings.tunneling.components.StaticContent
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.ui.theme.Typography
import net.nymtech.nymvpn.util.extensions.launchVpnSettings
import net.nymtech.nymvpn.util.extensions.safePopBackStack
import net.nymtech.nymvpn.util.extensions.scaledHeight
import net.nymtech.vpn.backend.Tunnel

@Composable
internal fun SplitTunnelingScreen(
	onBackEventConsume: () -> Unit,
	onBackClickEventTriggered: Boolean = false,
	navBarEvent: NavBarEvent?,
	onNavBarEventConsume: () -> Unit,
	viewModel: SplitTunnelingViewModel = hiltViewModel(),
) {
	val uiState by viewModel.uiState.collectAsStateWithLifecycle()
	val backendUi by viewModel.backendUi.collectAsStateWithLifecycle()
	val navController = LocalNavController.current
	val context = LocalContext.current

	val isActuallyConnected =
		backendUi.tunnelState == Tunnel.State.Up ||
			backendUi.tunnelState == Tunnel.State.EstablishingConnection

	val connectedForUi =
		backendUi.isRestarting ||
			backendUi.tunnelState == Tunnel.State.Up ||
			backendUi.tunnelState == Tunnel.State.InitializingClient ||
			backendUi.tunnelState == Tunnel.State.EstablishingConnection

	var showSplitTunnelingModal by remember { mutableStateOf(false) }

	LaunchedEffect(navBarEvent) {
		if (navBarEvent == NavBarEvent.SplitTunnelingInfoClicked) {
			showSplitTunnelingModal = true
			onNavBarEventConsume()
		}
	}

	SplitTunnelingInfoModal(
		showModal = showSplitTunnelingModal,
		onDismiss = { showSplitTunnelingModal = false },
	)

	LaunchedEffect(viewModel) {
		viewModel.events.collectLatest { event ->
			when (event) {
				UiEvent.ReconnectStarted ->
					Toast.makeText(context, context.getString(R.string.split_tunneling_event_reconnecting), Toast.LENGTH_SHORT).show()
			}
		}
	}

	val onNavigateBack = remember {
		{
			onBackEventConsume()
			navController.safePopBackStack()
		}
	}

	BackHandler {
		viewModel.requestBack()
	}

	LaunchedEffect(Unit) {
		viewModel.loadData()
	}

	LaunchedEffect(onBackClickEventTriggered) {
		if (onBackClickEventTriggered) viewModel.requestBack()
	}

	LaunchedEffect(uiState.navigateBack) {
		if (uiState.navigateBack) {
			viewModel.consumeNavigateBack()
			onNavigateBack()
		}
	}

	SplitTunnelingContent(
		uiState = uiState,
		connectedForUi = connectedForUi,
		onQueryChange = viewModel::onQueryChange,
		onSelectAllDirectAppsClick = viewModel::onSelectAllDirectAppsClick,
		onSelectAllVpnPassThroughClick = viewModel::onSelectAllVpnPassThroughClick,
		onChangeSelection = viewModel::onChangeSelection,
		onSave = {
			viewModel.saveChangesAndMaybeReconnect(isActuallyConnected)
			if (!isActuallyConnected) {
				Toast.makeText(context, context.getString(R.string.split_tunneling_event_saved), Toast.LENGTH_SHORT).show()
			}
		},
	)

	SaveChangesModal(
		showSaveChangesDialog = uiState.showSaveChangesDialog,
		confirmTextResId = if (connectedForUi) R.string.dns_custom_button_save_reconnect else R.string.dns_custom_button_save,
		onClickSave = {
			viewModel.saveChangesAndMaybeReconnect(isActuallyConnected)
			viewModel.consumeNavigateBack()
			onNavigateBack()
		},
		onDiscard = {
			viewModel.discardAndNavigateBack()
		},
		onDismiss = {
			viewModel.clearSaveDialog()
		},
	)

	if (uiState.isLoading) {
		LoadingDialog()
	}
}

@Composable
private fun LockdownStateNotice(lockdownState: LockdownState) {
	val context = LocalContext.current

	when (lockdownState) {
		LockdownState.ACTIVE_STEERING -> {
			Card(
				shape = RoundedCornerShape(8.dp),
				colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.primaryContainer),
				modifier = Modifier.fillMaxWidth(),
			) {
				Text(
					text = stringResource(R.string.split_tunnel_lockdown_active_note),
					style = MaterialTheme.typography.bodyMedium,
					color = MaterialTheme.colorScheme.onPrimaryContainer,
					modifier = Modifier.padding(16.dp.scaledHeight()),
				)
			}
		}
		LockdownState.UNSUPPORTED_API -> {
			WarningCard(stringResource(R.string.split_tunnel_lockdown_legacy_warning))
			Spacer(modifier = Modifier.height(12.dp.scaledHeight()))
			MainStyledButton(
				onClick = { context.launchVpnSettings() },
				content = {
					Text(
						stringResource(R.string.split_tunnel_open_vpn_settings),
						style = MaterialTheme.typography.titleMedium,
					)
				},
				textColor = MaterialTheme.colorScheme.onPrimary,
				modifier = Modifier.fillMaxWidth().height(40.dp.scaledHeight()),
				shape = RoundedCornerShape(12.dp),
			)
		}
		LockdownState.OFF -> Unit
	}
}

@Composable
private fun SplitTunnelingContent(
	uiState: SplitTunnelingUiState,
	connectedForUi: Boolean,
	onQueryChange: (String) -> Unit,
	onSelectAllDirectAppsClick: () -> Unit,
	onSelectAllVpnPassThroughClick: () -> Unit,
	onChangeSelection: (String) -> Unit,
	onSave: () -> Unit,
) {
	val interactionSource = remember { MutableInteractionSource() }

	val saveTextRes = if (connectedForUi) R.string.dns_custom_button_save_reconnect else R.string.dns_custom_button_save

	Box(modifier = Modifier.fillMaxSize()) {
		LazyColumn(
			modifier = Modifier
				.fillMaxSize()
				.windowInsetsPadding(WindowInsets.navigationBars)
				.imePadding(),
			contentPadding = PaddingValues(
				start = 16.dp.scaledHeight(),
				end = 16.dp.scaledHeight(),
				top = 16.dp.scaledHeight(),
				bottom = 120.dp.scaledHeight(),
			),
		) {
			if (uiState.lockdownState != LockdownState.OFF) {
				item {
					LockdownStateNotice(lockdownState = uiState.lockdownState)
					Spacer(modifier = Modifier.height(12.dp.scaledHeight()))
				}
			}

			item {
				StaticContent(
					uiState = uiState,
					onQueryChange = onQueryChange,
					onSelectAllDirectAppsClick = onSelectAllDirectAppsClick,
					onSelectAllVpnPassThroughClick = onSelectAllVpnPassThroughClick,
				)
			}

			items(
				items = uiState.filteredNormalApps,
				key = { app -> app.packageName },
			) { app ->
				Spacer(modifier = Modifier.height(12.dp.scaledHeight()))
				AppInfoRow(
					appInfo = app,
					onTogglePassThrough = onChangeSelection,
					mutableInteraction = interactionSource,
				)
				Spacer(modifier = Modifier.height(12.dp.scaledHeight()))
			}

			if (uiState.filteredSystemApps.isNotEmpty()) {
				item {
					Spacer(modifier = Modifier.height(12.dp.scaledHeight()))
					Text(
						text = stringResource(R.string.split_tunneling_system_applications),
						style = Typography.bodyMedium,
						color = MaterialTheme.colorScheme.onBackground,
					)
					Spacer(modifier = Modifier.height(12.dp.scaledHeight()))
				}

				items(
					items = uiState.filteredSystemApps,
					key = { app -> app.packageName },
				) { app ->
					Spacer(modifier = Modifier.height(12.dp.scaledHeight()))
					AppInfoRow(
						appInfo = app,
						onTogglePassThrough = onChangeSelection,
						mutableInteraction = interactionSource,
					)
					Spacer(modifier = Modifier.height(12.dp.scaledHeight()))
				}
			}
		}

		Box(
			modifier = Modifier
				.align(Alignment.BottomCenter)
				.fillMaxWidth()
				.background(MaterialTheme.colorScheme.background)
				.navigationBarsPadding()
				.padding(horizontal = 16.dp.scaledHeight(), vertical = 16.dp.scaledHeight()),
		) {
			MainStyledButton(
				onClick = onSave,
				enabled = uiState.hasUnsavedChanges,
				content = {
					Text(
						stringResource(saveTextRes),
						style = MaterialTheme.typography.titleMedium,
					)
				},
				textColor = MaterialTheme.colorScheme.onPrimary,
				modifier = Modifier.fillMaxWidth().height(48.dp.scaledHeight()),
				shape = RoundedCornerShape(12.dp),
			)
		}
	}
}

@PreviewLightDark
@Composable
internal fun SplitTunnelingPreview() {
	NymVPNTheme(Theme.default()) {
		Surface {
			SplitTunnelingContent(
				uiState = SplitTunnelingUiState(
					isLoading = false,
					filteredNormalApps = listOf(
						AppInfo(
							name = "App 1",
							icon = R.drawable.ic_launcher_foreground,
							packageName = "net.nymtech.nymvpn",
							passThroughVpn = false,
						),
						AppInfo(
							name = "App 2",
							icon = R.drawable.ic_launcher_foreground,
							packageName = "net.nymtech.nymvpn14",
						),
					),
					filteredSystemApps = listOf(
						AppInfo(
							name = "App 3",
							icon = R.drawable.ic_launcher_foreground,
							packageName = "net.nymtech.nymvpn3",
						),
					),
					directAppsCount = 3,
					vpnPassThroughAppsCount = 5,
					hasUnsavedChanges = true,
				),
				connectedForUi = true,
				onQueryChange = {},
				onSelectAllDirectAppsClick = {},
				onSelectAllVpnPassThroughClick = {},
				onChangeSelection = {},
				onSave = {},
			)
		}
	}
}
