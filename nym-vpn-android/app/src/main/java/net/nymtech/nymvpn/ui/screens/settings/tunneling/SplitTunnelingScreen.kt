package net.nymtech.nymvpn.ui.screens.settings.tunneling

import androidx.activity.compose.BackHandler
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.PaddingValues
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.navigationBars
import androidx.compose.foundation.layout.windowInsetsPadding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.PreviewLightDark
import androidx.compose.ui.unit.dp
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.AppUiState
import net.nymtech.nymvpn.ui.Route
import net.nymtech.nymvpn.ui.common.navigation.LocalNavController
import net.nymtech.nymvpn.ui.screens.settings.tunneling.components.AppInfoRow
import net.nymtech.nymvpn.ui.screens.settings.tunneling.components.LoadingDialog
import net.nymtech.nymvpn.ui.screens.settings.tunneling.components.SplitTunnelingSettingModal
import net.nymtech.nymvpn.ui.screens.settings.tunneling.components.StaticContent
import net.nymtech.nymvpn.ui.theme.LocalCustomColorsPalette
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.ui.theme.Typography
import net.nymtech.nymvpn.util.extensions.safePopBackStack
import net.nymtech.nymvpn.util.extensions.scaledHeight

@Composable
internal fun SplitTunnelingScreen(appState: AppUiState, onBackEventConsume: () -> Unit, onBackClickEventTriggered: Boolean = false, viewModel: SplitTunnelingViewModel = hiltViewModel()) {
	val uiState by viewModel.uiState.collectAsStateWithLifecycle()
	val navController = LocalNavController.current

	val onNavigateBack = remember {
		{
			onBackEventConsume()
			viewModel.onUiEvent(UiEvent.ClearNavigation)
			navController.safePopBackStack()
		}
	}
	val onNavigateHome = remember {
		{
			onBackEventConsume()
			viewModel.onUiEvent(UiEvent.ClearNavigation)
			val route = Route.Main()
			navController.navigate(route = route) {
				popUpTo(route) {
					inclusive = true
				}
				launchSingleTop = true
			}
		}
	}
	val onApplyNowClick = remember {
		{
			viewModel.disconnect()
		}
	}
	val onNextConnectionApplyClick = remember {
		{
			viewModel.onUiEvent(UiEvent.ClearDialog)
			viewModel.onUiEvent(UiEvent.NavigateBack)
		}
	}

	BackHandler {
		viewModel.onUiEvent(UiEvent.OnBackClick(appState.managerState.tunnelState))
	}

	LaunchedEffect(Unit) {
		viewModel.onUiEvent(UiEvent.LoadData)
	}
	LaunchedEffect(onBackClickEventTriggered) {
		if (onBackClickEventTriggered) {
			viewModel.onUiEvent(UiEvent.OnBackClick(appState.managerState.tunnelState))
		}
	}

	LaunchedEffect(uiState.pendingNavigation) {
		handleNavigation(
			pendingNavigation = uiState.pendingNavigation,
			onNavigateBack = onNavigateBack,
			onNavigateHome = onNavigateHome,
		)
	}

	uiState.pendingDialog?.let {
		when (it) {
			SplitTunnelingUiState.PendingDialog.AppListChangeDialog -> SplitTunnelingSettingModal(
				showModal = true,
				onApplyNowClick = onApplyNowClick,
				onNextConnectionApplyClick = onNextConnectionApplyClick,
			)
		}
	}

	SplitTunnelingScreen(uiState, viewModel::onUiEvent)
}

@Composable
private fun SplitTunnelingScreen(uiState: SplitTunnelingUiState, onUiEvent: (UiEvent) -> Unit) {
	val customColorPalette = LocalCustomColorsPalette.current
	val interactionSource = remember { MutableInteractionSource() }
	Box(modifier = Modifier.fillMaxSize()) {
		LazyColumn(
			modifier = Modifier
				.fillMaxSize()
				.windowInsetsPadding(WindowInsets.navigationBars)
				.imePadding(),
			contentPadding = PaddingValues(16.dp.scaledHeight()),
		) {
			item {
				StaticContent(uiState, onUiEvent)
			}

			items(
				items = uiState.filteredNormalApps,
				key = { app -> app.packageName },
			) { app ->
				Spacer(modifier = Modifier.height(12.dp.scaledHeight()))
				AppInfoRow(customColorPalette, app, onUiEvent, interactionSource)
				Spacer(modifier = Modifier.height(12.dp.scaledHeight()))
			}

			if (uiState.filteredSystemApps.isNotEmpty()) {
				item {
					Spacer(modifier = Modifier.height(12.dp.scaledHeight()))
					Text(
						text = stringResource(R.string.split_tunneling_system_applications),
						style = Typography.bodyMedium,
						color = MaterialTheme.colorScheme.outline,
					)
					Spacer(modifier = Modifier.height(12.dp.scaledHeight()))
				}

				items(
					items = uiState.filteredSystemApps,
					key = { app -> app.packageName },
				) { app ->
					Spacer(modifier = Modifier.height(12.dp.scaledHeight()))
					AppInfoRow(customColorPalette, app, onUiEvent, interactionSource)
					Spacer(modifier = Modifier.height(12.dp.scaledHeight()))
				}
			}
		}
		if (uiState.isLoading) {
			LoadingDialog()
		}
	}
}

private fun handleNavigation(pendingNavigation: SplitTunnelingUiState.PendingNavigation?, onNavigateBack: () -> Unit, onNavigateHome: () -> Unit) {
	pendingNavigation?.let {
		when (it) {
			SplitTunnelingUiState.PendingNavigation.NavigateBack -> onNavigateBack()
			SplitTunnelingUiState.PendingNavigation.NavigateToHome -> onNavigateHome()
		}
	}
}

@PreviewLightDark
@Composable
internal fun SplitTunnelingPreview() {
	NymVPNTheme(
		Theme.default(),
	) {
		Surface {
			SplitTunnelingScreen(
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
						AppInfo(
							name = "App 3",
							icon = R.drawable.ic_launcher_foreground,
							packageName = "net.nymtech.nymvpn13",
							passThroughVpn = false,
						),
						AppInfo(
							name = "App 4",
							icon = R.drawable.ic_launcher_foreground,
							packageName = "net.nymtech.nymvpn12",
							passThroughVpn = false,
						),
						AppInfo(
							name = "App 5",
							icon = R.drawable.ic_launcher_foreground,
							packageName = "net.nymtech.nymvpn11",
						),
					),
					filteredSystemApps = listOf(
						AppInfo(
							name = "App 1",
							icon = R.drawable.ic_launcher_foreground,
							packageName = "net.nymtech.nymvpn1",
						),
						AppInfo(
							name = "App 2",
							icon = R.drawable.ic_launcher_foreground,
							packageName = "net.nymtech.nymvpn2",
						),
						AppInfo(
							name = "App 3",
							icon = R.drawable.ic_launcher_foreground,
							packageName = "net.nymtech.nymvpn3",
						),
					),
					directAppsCount = 3,
					vpnPassThroughAppsCount = 5,
				),
				onUiEvent = {},
			)
		}
	}
}
