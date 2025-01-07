package net.nymtech.nymvpn.ui.screens.settings

import android.os.Build
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.asPaddingValues
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.systemBars
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.automirrored.outlined.ArrowRight
import androidx.compose.material.icons.automirrored.outlined.Launch
import androidx.compose.material.icons.automirrored.outlined.ViewQuilt
import androidx.compose.material.icons.outlined.AdminPanelSettings
import androidx.compose.material.icons.outlined.AppShortcut
import androidx.compose.material.icons.outlined.Notifications
import androidx.compose.material.icons.outlined.Person
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.platform.ClipboardManager
import androidx.compose.ui.platform.LocalClipboardManager
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.res.vectorResource
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import net.nymtech.nymvpn.BuildConfig
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.AppUiState
import net.nymtech.nymvpn.ui.AppViewModel
import net.nymtech.nymvpn.ui.Route
import net.nymtech.nymvpn.ui.common.buttons.MainStyledButton
import net.nymtech.nymvpn.ui.common.buttons.ScaledSwitch
import net.nymtech.nymvpn.ui.common.buttons.surface.SelectionItem
import net.nymtech.nymvpn.ui.common.buttons.surface.SurfaceSelectionGroupButton
import net.nymtech.nymvpn.ui.common.navigation.LocalNavController
import net.nymtech.nymvpn.ui.common.navigation.NavBarState
import net.nymtech.nymvpn.ui.common.navigation.NavIcon
import net.nymtech.nymvpn.ui.common.navigation.NavTitle
import net.nymtech.nymvpn.ui.common.snackbar.SnackbarController
import net.nymtech.nymvpn.ui.theme.CustomTypography
import net.nymtech.nymvpn.ui.theme.iconSize
import net.nymtech.nymvpn.util.extensions.launchNotificationSettings
import net.nymtech.nymvpn.util.extensions.launchVpnSettings
import net.nymtech.nymvpn.util.extensions.openWebUrl
import net.nymtech.nymvpn.util.extensions.scaledWidth
import net.nymtech.vpn.backend.Tunnel
import timber.log.Timber

@Composable
fun SettingsScreen(appViewModel: AppViewModel, appUiState: AppUiState, viewModel: SettingsViewModel = hiltViewModel()) {
	val context = LocalContext.current
	val snackbar = SnackbarController.current
	val navController = LocalNavController.current
	val clipboardManager: ClipboardManager = LocalClipboardManager.current
	val padding = WindowInsets.systemBars.asPaddingValues()

	LaunchedEffect(Unit) {
		appViewModel.onNavBarStateChange(
			NavBarState(
				title = { NavTitle(stringResource(R.string.settings)) },
				leading = {
					NavIcon(Icons.AutoMirrored.Filled.ArrowBack) {
						navController.popBackStack()
					}
				},
			),
		)
	}

	Column(
		horizontalAlignment = Alignment.Start,
		verticalArrangement = Arrangement.spacedBy(24.dp, Alignment.Top),
		modifier =
		Modifier
			.verticalScroll(rememberScrollState())
			.fillMaxSize()
			.padding(top = 24.dp)
			.padding(horizontal = 24.dp.scaledWidth()).padding(bottom = padding.calculateBottomPadding()),
	) {
		if (!appUiState.managerState.isMnemonicStored) {
			MainStyledButton(
				onClick = { navController.navigate(Route.Credential) },
				content = {
					Text(
						stringResource(id = R.string.log_in),
						style = CustomTypography.labelHuge,
					)
				},
				color = MaterialTheme.colorScheme.primary,
			)
		} else {
			SurfaceSelectionGroupButton(
				listOf(
					SelectionItem(
						Icons.Outlined.Person,
						{
							val icon = Icons.AutoMirrored.Outlined.Launch
							Icon(icon, icon.name, Modifier.size(iconSize))
						},
						title = { Text(stringResource(R.string.account), style = MaterialTheme.typography.bodyLarge.copy(MaterialTheme.colorScheme.onSurface)) },
						onClick = {
							appUiState.managerState.accountLinks?.account?.let {
								Timber.d("Account url: $it")
								context.openWebUrl(it)
							}
						},
					),
				),
			)
		}
		SurfaceSelectionGroupButton(
			listOf(
				SelectionItem(
					ImageVector.vectorResource(R.drawable.auto),
					{
						ScaledSwitch(
							appUiState.settings.autoStartEnabled,
							onClick = { viewModel.onAutoConnectSelected(it) },
						)
					},
					title = { Text(stringResource(R.string.auto_connect), style = MaterialTheme.typography.bodyLarge.copy(MaterialTheme.colorScheme.onSurface)) },
					description = {
						Text(
							stringResource(id = R.string.auto_connect_description),
							style = MaterialTheme.typography.bodyMedium.copy(MaterialTheme.colorScheme.outline),
						)
					},
				),
				SelectionItem(
					Icons.Outlined.AdminPanelSettings,
					{
						val icon = Icons.AutoMirrored.Outlined.ArrowRight
						Icon(icon, icon.name, Modifier.size(iconSize))
					},
					title = { Text(stringResource(R.string.kill_switch), style = MaterialTheme.typography.bodyLarge.copy(MaterialTheme.colorScheme.onSurface)) },
					onClick = {
						context.launchVpnSettings()
					},
				),
			),
		)
		SurfaceSelectionGroupButton(
			mutableListOf(
				SelectionItem(
					Icons.AutoMirrored.Outlined.ViewQuilt,
					{
						val icon = Icons.AutoMirrored.Outlined.ArrowRight
						Icon(icon, icon.name, Modifier.size(iconSize))
					},
					title = { Text(stringResource(R.string.appearance), style = MaterialTheme.typography.bodyLarge.copy(MaterialTheme.colorScheme.onSurface)) },
					onClick = { navController.navigate(Route.Appearance) },
				),
				SelectionItem(
					Icons.Outlined.Notifications,
					{
						val icon = Icons.AutoMirrored.Outlined.ArrowRight
						Icon(icon, icon.name, Modifier.size(iconSize))
					},
					title = { Text(stringResource(R.string.notifications), style = MaterialTheme.typography.bodyLarge.copy(MaterialTheme.colorScheme.onSurface)) },
					onClick = {
						context.launchNotificationSettings()
					},
				),
			).apply {
				if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.N_MR1) {
					this.add(
						SelectionItem(
							Icons.Outlined.AppShortcut,
							{
								ScaledSwitch(
									appUiState.settings.isShortcutsEnabled,
									onClick = { checked -> viewModel.onAppShortcutsSelected(checked) },
								)
							},
							title = { Text(stringResource(R.string.app_shortcuts), style = MaterialTheme.typography.bodyLarge.copy(MaterialTheme.colorScheme.onSurface)) },
							description = {
								Text(
									stringResource(id = R.string.enable_shortcuts),
									style = MaterialTheme.typography.bodyMedium.copy(MaterialTheme.colorScheme.outline),
								)
							},
						),
					)
				}
			},
		)
		SurfaceSelectionGroupButton(
			listOf(
				SelectionItem(
					ImageVector.vectorResource(R.drawable.support),
					{
						val icon = Icons.AutoMirrored.Outlined.ArrowRight
						Icon(icon, icon.name, Modifier.size(iconSize))
					},
					title = {
						Text(
							stringResource(R.string.support_and_feedback),
							style = MaterialTheme.typography.bodyLarge.copy(MaterialTheme.colorScheme.onSurface),
						)
					},
					onClick = { navController.navigate(Route.Support) },
				),
				SelectionItem(
					ImageVector.vectorResource(R.drawable.logs),
					{
						val icon = Icons.AutoMirrored.Outlined.ArrowRight
						Icon(icon, icon.name, Modifier.size(iconSize))
					},
					title = { Text(stringResource(R.string.logs), style = MaterialTheme.typography.bodyLarge.copy(MaterialTheme.colorScheme.onSurface)) },
					onClick = { navController.navigate(Route.Logs) },
				),
				// TODO disable until api ready
// 				SelectionItem(
// 					Icons.Outlined.Analytics,
// 					title = {
// 						Text(
// 							stringResource(R.string.anonymous_analytics),
// 							style = MaterialTheme.typography.bodyLarge.copy(MaterialTheme.colorScheme.onSurface),
// 						)
// 					},
// 					description = {
// 						Text(
// 							stringResource(id = R.string.anonymous_analytics_description),
// 							style = MaterialTheme.typography.bodyMedium.copy(MaterialTheme.colorScheme.outline),
// 						)
// 					},
// 					trailing = {
// 						ScaledSwitch(
// 							appUiState.settings.analyticsEnabled,
// 							{ appViewModel.onAnalyticsReportingSelected() },
// 							modifier =
// 							Modifier
// 								.height(32.dp.scaledHeight())
// 								.width(52.dp.scaledWidth()),
// 						)
// 					},
// 				),
			),
		)
		SurfaceSelectionGroupButton(
			listOf(
				SelectionItem(
					trailing = {
						val icon = Icons.AutoMirrored.Outlined.ArrowRight
						Icon(icon, icon.name, Modifier.size(iconSize))
					},
					title = { Text(stringResource(R.string.legal), style = MaterialTheme.typography.bodyLarge.copy(MaterialTheme.colorScheme.onSurface)) },
					onClick = { navController.navigate(Route.Legal) },
				),
			),
		)
		if (appUiState.managerState.isMnemonicStored) {
			SurfaceSelectionGroupButton(
				listOf(
					SelectionItem(
						title = { Text(stringResource(R.string.log_out), style = MaterialTheme.typography.bodyLarge.copy(MaterialTheme.colorScheme.onSurface)) },
						onClick = {
							if (appUiState.managerState.tunnelState == Tunnel.State.Down) {
								appViewModel.logout()
							} else {
								snackbar.showMessage(context.getString(R.string.action_requires_tunnel_down))
							}
						},
						trailing = {},
					),
				),
			)
		}
		Column(
			verticalArrangement = Arrangement.Bottom,
			horizontalAlignment = Alignment.Start,
			modifier =
			Modifier
				.fillMaxSize()
				.padding(bottom = 20.dp),
		) {
			Text(
				stringResource(R.string.version) + ": ${BuildConfig.VERSION_NAME}",
				style = MaterialTheme.typography.bodyMedium,
				color = MaterialTheme.colorScheme.secondary,
				modifier = Modifier.clickable {
					if (BuildConfig.DEBUG || BuildConfig.IS_PRERELEASE) {
						navController.navigate(Route.Developer)
					} else {
						clipboardManager.setText(
							annotatedString = AnnotatedString(BuildConfig.VERSION_NAME),
						)
					}
				},
			)
		}
	}
}
