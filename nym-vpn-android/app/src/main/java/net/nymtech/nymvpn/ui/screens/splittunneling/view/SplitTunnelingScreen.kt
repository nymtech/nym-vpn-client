package net.nymtech.nymvpn.ui.screens.splittunneling.view

import android.graphics.Bitmap
import androidx.activity.compose.BackHandler
import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.IntrinsicSize
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.navigationBars
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.widthIn
import androidx.compose.foundation.layout.windowInsetsPadding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Shield
import androidx.compose.material.icons.rounded.Search
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.asImageBitmap
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.painterResource
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.res.vectorResource
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.tooling.preview.PreviewLightDark
import androidx.compose.ui.unit.dp
import androidx.compose.ui.unit.sp
import androidx.core.graphics.drawable.toBitmapOrNull
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.AppUiState
import net.nymtech.nymvpn.ui.Route
import net.nymtech.nymvpn.ui.common.VerticalDivider
import net.nymtech.nymvpn.ui.common.navigation.LocalNavController
import net.nymtech.nymvpn.ui.common.textbox.CustomTextField
import net.nymtech.nymvpn.ui.screens.splittunneling.model.AppFilter
import net.nymtech.nymvpn.ui.screens.splittunneling.model.AppInfo
import net.nymtech.nymvpn.ui.screens.splittunneling.model.SplitTunnelingUiState
import net.nymtech.nymvpn.ui.screens.splittunneling.model.UiEvent
import net.nymtech.nymvpn.ui.screens.splittunneling.view.components.SplitTunnelingSettingModal
import net.nymtech.nymvpn.ui.screens.splittunneling.viewmodel.SplitTunnelingViewModel
import net.nymtech.nymvpn.ui.theme.CustomColors
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.ui.theme.Typography
import net.nymtech.nymvpn.ui.theme.iconSize
import net.nymtech.nymvpn.util.extensions.safePopBackStack
import net.nymtech.nymvpn.util.extensions.scaledHeight
import net.nymtech.nymvpn.util.extensions.scaledWidth

@Composable
internal fun SplitTunnelingScreen(
	appState: AppUiState,
	onBackEventConsumed: () -> Unit,
	onBackClickEventTriggered: Boolean = false,
	viewModel: SplitTunnelingViewModel = hiltViewModel(),
) {
	val uiState by viewModel.uiState.collectAsStateWithLifecycle()
	val navController = LocalNavController.current

	val onNavigateBack = remember {
		{
			onBackEventConsumed()
			viewModel.onUiEvent(UiEvent.ClearNavigation)
			navController.safePopBackStack()
		}
	}
	val onNavigateHome = remember {
		{
			onBackEventConsumed()
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

	LaunchedEffect(onBackClickEventTriggered) {
		if (onBackClickEventTriggered) {
			viewModel.onUiEvent(UiEvent.OnBackClick(appState.managerState.tunnelState))
		}
	}

	LaunchedEffect(uiState.pendingNavigation) {
		handleNavigation(
			pendingNavigation = uiState.pendingNavigation,
			onNavigateBack = onNavigateBack,
			onNavigateHome = onNavigateHome
		)
	}

	HandleDialog(
		pendingDialog = uiState.pendingDialog,
		onApplyNowClick = onApplyNowClick,
		onNextConnectionApplyClick = onNextConnectionApplyClick,
	)

	SplitTunneling(uiState, viewModel::onUiEvent)
}

@Composable
private fun SplitTunneling(uiState: SplitTunnelingUiState, onUiEvent: (UiEvent) -> Unit) {
	Column(
		modifier = Modifier
			.fillMaxSize()
			.verticalScroll(rememberScrollState())
			.windowInsetsPadding(WindowInsets.navigationBars)
			.imePadding()
			.padding(16.dp.scaledHeight()),
	) {
		Text(
			text = stringResource(R.string.split_tunneling_info_msg),
			style = Typography.bodyMedium,
			color = MaterialTheme.colorScheme.outline,
			modifier = Modifier.padding(top = 8.dp.scaledHeight()),
			fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
		)
		Text(
			text = stringResource(R.string.split_tunneling_info_desc),
			modifier = Modifier.padding(top = 12.dp.scaledHeight()),
			style = Typography.bodyMedium,
			color = CustomColors.warning,
			fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
		)
		CustomTextField(
			value = uiState.query,
			onValueChange = { onUiEvent(UiEvent.QueryChange(it)) },
			modifier = Modifier
				.fillMaxWidth()
				.padding(vertical = 24.dp.scaledHeight())
				.height(56.dp.scaledHeight())
				.background(Color.Transparent, RoundedCornerShape(30.dp)),
			placeholder = { Text(stringResource(R.string.search_apps_hint), color = MaterialTheme.colorScheme.outline) },
			singleLine = true,
			leading = { Icon(Icons.Rounded.Search, contentDescription = stringResource(R.string.search), modifier = Modifier.size(iconSize)) },
			label = { Text(stringResource(R.string.search)) },
			textStyle = MaterialTheme.typography.bodyLarge.copy(color = MaterialTheme.colorScheme.onSurface),
		)
		Text(
			text = stringResource(R.string.apps),
			style = Typography.bodyLarge,
			color = MaterialTheme.colorScheme.onSurface,
			fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
			fontWeight = FontWeight(500),
		)

		Row(
			modifier = Modifier
				.padding(top = 12.dp.scaledHeight(), bottom = 24.dp.scaledHeight())
				.fillMaxWidth()
				.height(IntrinsicSize.Min),
			horizontalArrangement = Arrangement.spacedBy(12.dp.scaledWidth()),
			verticalAlignment = Alignment.CenterVertically,
		) {
			FilterButton(
				stringResource(R.string.direct),
				uiState.directAppsCount,
				stringResource(R.string.direct_desc),
				ImageVector.vectorResource(R.drawable.split),
				isSelected = uiState.appliedFilter == AppFilter.Direct,
				modifier = Modifier
					.weight(1f)
					.fillMaxHeight()
					.clickable {
						onUiEvent(UiEvent.SelectAllDirectAppsClick)
					},
			)
			FilterButton(
				stringResource(R.string.via_vpn),
				uiState.vpnPassThroughAppsCount,
				stringResource(R.string.via_desc),
				Icons.Filled.Shield,
				isSelected = uiState.appliedFilter == AppFilter.VpnPassThrough,
				modifier = Modifier
					.weight(1f)
					.fillMaxHeight()
					.clickable {
						onUiEvent(UiEvent.SelectAllVpnPassThroughClick)
					},
			)
		}

		Row(
			modifier = Modifier
				.fillMaxWidth()
				.padding(bottom = 12.dp.scaledHeight()),
			horizontalArrangement = Arrangement.End,
		) {
			Text(
				text = stringResource(R.string.direct),
				modifier = Modifier.widthIn(min = 42.dp),
				style = Typography.bodySmall.copy(fontSize = 10.sp, lineHeight = 14.sp),
				color = MaterialTheme.colorScheme.onSurface,
				textAlign = TextAlign.Center,
			)
			Text(
				text = stringResource(R.string.nym_vpn),
				modifier = Modifier.widthIn(min = 42.dp),
				style = Typography.bodySmall.copy(fontSize = 10.sp, lineHeight = 14.sp),
				color = MaterialTheme.colorScheme.onSurface,
				textAlign = TextAlign.Center,
			)
		}

		HorizontalDivider(modifier = Modifier.fillMaxWidth(), color = MaterialTheme.colorScheme.surface.copy(alpha = 0.1f))

		uiState.filteredNormalApps.forEach { app ->
			Spacer(modifier = Modifier.padding(bottom = 12.dp.scaledHeight()))
			AppInfoRow(app, onUiEvent)
			Spacer(modifier = Modifier.padding(bottom = 12.dp.scaledHeight()))
		}
		Spacer(modifier = Modifier.padding(bottom = 14.dp.scaledHeight()))
		if (uiState.filteredSystemApps.isNotEmpty()) {
			Text(
				text = stringResource(R.string.system_applications),
				style = Typography.bodyMedium,
				color = MaterialTheme.colorScheme.outline,
			)
		}
		Spacer(modifier = Modifier.padding(bottom = 14.dp.scaledHeight()))
		uiState.filteredSystemApps.forEach { app ->
			Spacer(modifier = Modifier.padding(bottom = 12.dp.scaledHeight()))
			AppInfoRow(app, onUiEvent)
			Spacer(modifier = Modifier.padding(bottom = 12.dp.scaledHeight()))
		}
	}
}

@Composable
private fun FilterButton(
	title: String,
	noOfApps: Int,
	description: String,
	imageVector: ImageVector,
	isSelected: Boolean,
	modifier: Modifier = Modifier,
) {
	Card(
		modifier = modifier,
		shape = RoundedCornerShape(8.dp.scaledHeight()),
		colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.surface),
		border = if (isSelected) BorderStroke(1.dp, MaterialTheme.colorScheme.onBackground) else null,
	) {
		Column(
			verticalArrangement = Arrangement.Center,
			modifier = Modifier.padding(horizontal = 12.dp.scaledWidth(), vertical = 8.dp.scaledHeight()),
		) {
			Row(verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(4.dp.scaledWidth())) {
				Icon(
					imageVector = imageVector,
					contentDescription = null,
					modifier = Modifier.size(16.dp.scaledHeight()),
				)
				Text(
					text = title,
					style = Typography.bodyMedium.copy(fontWeight = if (isSelected) FontWeight(700) else Typography.bodyMedium.fontWeight),
					color = MaterialTheme.colorScheme.onBackground,
					fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
				)
				Text(
					text = "($noOfApps)",
					style = Typography.bodySmall,
					color = MaterialTheme.colorScheme.outline,
				)
			}
			Spacer(modifier = Modifier.height(4.dp.scaledHeight()))
			Text(
				text = description,
				style = Typography.bodySmall.copy(fontSize = 10.sp, lineHeight = 14.sp),
				color = if (isSelected) MaterialTheme.colorScheme.onBackground else MaterialTheme.colorScheme.outline,
			)
		}
	}
}

@Composable
private fun AppInfoRow(appInfo: AppInfo, onUiEvent: (UiEvent) -> Unit) {
	Row(
		modifier = Modifier.fillMaxWidth(),
		verticalAlignment = Alignment.CenterVertically,
	) {
		loadIcon(appInfo.packageName)?.let {
			Icon(
				it.asImageBitmap(),
				contentDescription = appInfo.name,
				tint = Color.Unspecified,
				modifier = Modifier
					.padding(end = 16.dp.scaledHeight())
					.size(iconSize.scaledHeight()),
			)
		}
		Text(
			text = appInfo.name,
			style = Typography.bodyMedium,
			color = MaterialTheme.colorScheme.onBackground,
			fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
			modifier = Modifier.weight(1f),
		)
		Row(
			modifier = Modifier
				.border(width = 1.dp, color = MaterialTheme.colorScheme.outline, shape = RoundedCornerShape(8.dp.scaledHeight()))
				.clickable {
					onUiEvent(UiEvent.ChangeSelection(appInfo.packageName))
				},
			verticalAlignment = Alignment.CenterVertically,
		) {
			Box(
				modifier = Modifier
					.then(
						if (!appInfo.passThroughVpn) {
							Modifier.background(
								CustomColors.error.copy(alpha = 0.1f),
								shape = RoundedCornerShape(topStart = 8.dp.scaledHeight(), bottomStart = 8.dp.scaledHeight()),
							)
						} else {
							Modifier
						},
					)
					.padding(start = 8.dp.scaledWidth(), end = 2.dp.scaledWidth()),
			) {
				Icon(
					painterResource(R.drawable.split),
					contentDescription = null,
					modifier = Modifier
						.padding(horizontal = 8.dp.scaledHeight(), vertical = 4.dp.scaledHeight())
						.size(16.dp.scaledHeight()),
					tint = if (!appInfo.passThroughVpn) CustomColors.error else MaterialTheme.colorScheme.outline,
				)
			}
			VerticalDivider(modifier = Modifier.height(24.dp))
			Box(
				modifier = Modifier
					.then(
						if (appInfo.passThroughVpn) {
							Modifier.background(
								MaterialTheme.colorScheme.primary.copy(alpha = 0.1f),
								shape = RoundedCornerShape(topEnd = 8.dp.scaledHeight(), bottomEnd = 8.dp.scaledHeight()),
							)
						} else {
							Modifier
						},
					)
					.padding(start = 2.dp.scaledWidth(), end = 8.dp.scaledWidth()),
			) {
				Icon(
					Icons.Filled.Shield,
					contentDescription = null,
					modifier = Modifier
						.padding(horizontal = 8.dp.scaledHeight(), vertical = 4.dp.scaledHeight())
						.size(16.dp.scaledHeight()),
					tint = if (appInfo.passThroughVpn) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.outline,
				)
			}
		}
	}
}

@Composable
private fun loadIcon(packageName: String): Bitmap? {
	val context = LocalContext.current
	val packageManager = remember(context) { context.packageManager }
	return try {
		packageManager.getApplicationIcon(packageName).toBitmapOrNull()
	} catch (_: Exception) {
		null
	}
}
private fun handleNavigation(
	pendingNavigation: SplitTunnelingUiState.PendingNavigation?,
	onNavigateBack: () -> Unit,
	onNavigateHome: () -> Unit,
) {
	pendingNavigation?.let {
		when (it) {
			SplitTunnelingUiState.PendingNavigation.NavigateBack -> onNavigateBack()
			SplitTunnelingUiState.PendingNavigation.NavigateToHome -> onNavigateHome()
		}
	}
}

@Composable
private fun HandleDialog(pendingDialog: SplitTunnelingUiState.PendingDialog?, onApplyNowClick: () -> Unit, onNextConnectionApplyClick: () -> Unit) {
	pendingDialog?.let {
		when (it) {
			SplitTunnelingUiState.PendingDialog.AppListChangeDialog -> SplitTunnelingSettingModal(
				showModal = true,
				onApplyNowClick = onApplyNowClick,
				onNextConnectionApplyClick = onNextConnectionApplyClick,
			)
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
			SplitTunneling(
				uiState = SplitTunnelingUiState(
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
							packageName = "net.nymtech.nymvpn",
						),
						AppInfo(
							name = "App 3",
							icon = R.drawable.ic_launcher_foreground,
							packageName = "net.nymtech.nymvpn",
							passThroughVpn = false,
						),
						AppInfo(
							name = "App 4",
							icon = R.drawable.ic_launcher_foreground,
							packageName = "net.nymtech.nymvpn",
							passThroughVpn = false,
						),
						AppInfo(
							name = "App 5",
							icon = R.drawable.ic_launcher_foreground,
							packageName = "net.nymtech.nymvpn",
						),
					),
					filteredSystemApps = listOf(
						AppInfo(
							name = "App 1",
							icon = R.drawable.ic_launcher_foreground,
							packageName = "net.nymtech.nymvpn",
						),
						AppInfo(
							name = "App 2",
							icon = R.drawable.ic_launcher_foreground,
							packageName = "net.nymtech.nymvpn",
						),
						AppInfo(
							name = "App 3",
							icon = R.drawable.ic_launcher_foreground,
							packageName = "net.nymtech.nymvpn",
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
