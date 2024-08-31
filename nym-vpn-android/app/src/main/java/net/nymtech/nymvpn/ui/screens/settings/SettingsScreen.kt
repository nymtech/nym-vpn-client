package net.nymtech.nymvpn.ui.screens.settings

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.asPaddingValues
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.systemBars
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.text.ClickableText
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.outlined.ViewQuilt
import androidx.compose.material.icons.filled.AccountCircle
import androidx.compose.material.icons.outlined.AdminPanelSettings
import androidx.compose.material.icons.outlined.AppShortcut
import androidx.compose.material.icons.outlined.BugReport
import androidx.compose.material.icons.outlined.Notifications
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.platform.ClipboardManager
import androidx.compose.ui.platform.LocalClipboardManager
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.res.vectorResource
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.withStyle
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import net.nymtech.nymvpn.BuildConfig
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.AppUiState
import net.nymtech.nymvpn.ui.AppViewModel
import net.nymtech.nymvpn.ui.Destination
import net.nymtech.nymvpn.ui.common.buttons.MainStyledButton
import net.nymtech.nymvpn.ui.common.buttons.ScaledSwitch
import net.nymtech.nymvpn.ui.common.buttons.surface.SelectionItem
import net.nymtech.nymvpn.ui.common.buttons.surface.SurfaceSelectionGroupButton
import net.nymtech.nymvpn.ui.theme.CustomTypography
import net.nymtech.nymvpn.util.extensions.durationFromNow
import net.nymtech.nymvpn.util.extensions.go
import net.nymtech.nymvpn.util.extensions.isInvalid
import net.nymtech.nymvpn.util.extensions.launchNotificationSettings
import net.nymtech.nymvpn.util.extensions.launchVpnSettings
import net.nymtech.nymvpn.util.extensions.openWebUrl
import net.nymtech.nymvpn.util.extensions.scaledHeight
import net.nymtech.nymvpn.util.extensions.scaledWidth
import net.nymtech.vpn.backend.Tunnel

@Composable
fun SettingsScreen(appViewModel: AppViewModel, appUiState: AppUiState, viewModel: SettingsViewModel = hiltViewModel()) {
	val context = LocalContext.current
	val clipboardManager: ClipboardManager = LocalClipboardManager.current
	val navController = appViewModel.navController
	val padding = WindowInsets.systemBars.asPaddingValues()

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
		if (appUiState.settings.credentialExpiry.isInvalid()) {
			MainStyledButton(
				onClick = { navController.go(Destination.Credential.route) },
				content = {
					Text(
						stringResource(id = R.string.add_cred_to_connect),
						style = CustomTypography.labelHuge,
					)
				},
				color = MaterialTheme.colorScheme.primary,
			)
		} else {
			appUiState.settings.credentialExpiry?.let {
				val credentialDuration = it.durationFromNow()
				val days = credentialDuration.toDaysPart()
				val hours = credentialDuration.toHoursPart()
				val accountDescription =
					buildAnnotatedString {
						if (days != 0L) {
							append(days.toString())
							append(" ")
							append(if (days != 1L) stringResource(id = R.string.days) else stringResource(id = R.string.day))
						} else {
							append(hours.toString())
							append(" ")
							append(if (hours != 1) stringResource(id = R.string.hours) else stringResource(id = R.string.hour))
						}
						append(" ")
						append(stringResource(id = R.string.remaining))
					}
				SurfaceSelectionGroupButton(
					listOf(
						SelectionItem(
							Icons.Filled.AccountCircle,
							onClick = {
								navController.go(Destination.Account.route)
							},
							title = { Text(stringResource(R.string.credential), style = MaterialTheme.typography.bodyLarge.copy(MaterialTheme.colorScheme.onSurface)) },
							description = { Text(accountDescription.text, style = MaterialTheme.typography.bodyMedium.copy(MaterialTheme.colorScheme.outline)) },
						),
					),
				)
			}
		}
		SurfaceSelectionGroupButton(
			listOf(
				SelectionItem(
					ImageVector.vectorResource(R.drawable.auto),
					{
						ScaledSwitch(
							appUiState.settings.autoStartEnabled,
							onClick = { viewModel.onAutoConnectSelected(it) },
							modifier =
							Modifier
								.height(32.dp.scaledHeight())
								.width(52.dp.scaledWidth()),
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
					title = { Text(stringResource(R.string.kill_switch), style = MaterialTheme.typography.bodyLarge.copy(MaterialTheme.colorScheme.onSurface)) },
					onClick = {
						context.launchVpnSettings()
					},
				),
				SelectionItem(
					Icons.Outlined.AppShortcut,
					{
						ScaledSwitch(
							appUiState.settings.isShortcutsEnabled,
							onClick = { viewModel.onAppShortcutsSelected(it) },
							modifier =
							Modifier
								.height(32.dp.scaledHeight())
								.width(52.dp.scaledWidth()),
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
				SelectionItem(
					ImageVector.vectorResource(R.drawable.two),
					{
						ScaledSwitch(
							appUiState.settings.firstHopSelectionEnabled,
							onClick = { appViewModel.onEntryLocationSelected(it) },
							modifier =
							Modifier
								.height(32.dp.scaledHeight())
								.width(52.dp.scaledWidth()),
							enabled = (appUiState.state is Tunnel.State.Down),
						)
					},
					title = {
						Text(
							stringResource(R.string.entry_location_selector),
							style = MaterialTheme.typography.bodyLarge.copy(MaterialTheme.colorScheme.onSurface),
						)
					},
				),
			),
		)
		SurfaceSelectionGroupButton(
			listOf(
				SelectionItem(
					Icons.AutoMirrored.Outlined.ViewQuilt,
					title = { Text(stringResource(R.string.appearance), style = MaterialTheme.typography.bodyLarge.copy(MaterialTheme.colorScheme.onSurface)) },
					onClick = { navController.go(Destination.Appearance.route) },
				),
				SelectionItem(
					Icons.Outlined.Notifications,
					title = { Text(stringResource(R.string.notifications), style = MaterialTheme.typography.bodyLarge.copy(MaterialTheme.colorScheme.onSurface)) },
					onClick = {
						context.launchNotificationSettings()
					},
				),
			),
		)
		val errorReportingDescription = buildAnnotatedString {
			append("(")
			append(stringResource(id = R.string.via))
			append(" ")
			pushStringAnnotation(tag = "sentry", annotation = stringResource(id = R.string.sentry_url))
			withStyle(style = SpanStyle(color = MaterialTheme.colorScheme.primary)) {
				append(stringResource(id = R.string.sentry))
			}
			pop()
			append("), ")
			append(stringResource(id = R.string.required_app_restart))
		}
		SurfaceSelectionGroupButton(
			listOf(
				SelectionItem(
					ImageVector.vectorResource(R.drawable.feedback),
					title = { Text(stringResource(R.string.feedback), style = MaterialTheme.typography.bodyLarge.copy(MaterialTheme.colorScheme.onSurface)) },
					onClick = { navController.go(Destination.Feedback.route) },
				),
				SelectionItem(
					ImageVector.vectorResource(R.drawable.support),
					title = { Text(stringResource(R.string.support), style = MaterialTheme.typography.bodyLarge.copy(MaterialTheme.colorScheme.onSurface)) },
					onClick = { navController.go(Destination.Support.route) },
				),
				SelectionItem(
					ImageVector.vectorResource(R.drawable.logs),
					title = { Text(stringResource(R.string.logs), style = MaterialTheme.typography.bodyLarge.copy(MaterialTheme.colorScheme.onSurface)) },
					onClick = { navController.go(Destination.Logs.route) },
				),
				SelectionItem(
					Icons.Outlined.BugReport,
					title = {
						Text(stringResource(R.string.anonymous_error_reports), style = MaterialTheme.typography.bodyLarge.copy(MaterialTheme.colorScheme.onSurface))
					},
					description = {
						ClickableText(
							text = errorReportingDescription,
							style = MaterialTheme.typography.bodyMedium.copy(color = MaterialTheme.colorScheme.onSurfaceVariant),
						) {
							errorReportingDescription.getStringAnnotations(tag = "sentry", it, it).firstOrNull()?.let { annotation ->
								context.openWebUrl(annotation.item)
							}
						}
					},
					trailing = {
						ScaledSwitch(
							checked = appUiState.settings.errorReportingEnabled,
							onClick = { appViewModel.onErrorReportingSelected() },
							modifier =
							Modifier
								.height(32.dp.scaledHeight())
								.width(52.dp.scaledWidth()),
						)
					},
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
					title = { Text(stringResource(R.string.legal), style = MaterialTheme.typography.bodyLarge.copy(MaterialTheme.colorScheme.onSurface)) },
					onClick = { navController.go(Destination.Legal.route) },
				),
			),
		)
		Column(
			verticalArrangement = Arrangement.Bottom,
			horizontalAlignment = Alignment.Start,
			modifier =
			Modifier
				.fillMaxSize()
				.padding(bottom = 20.dp),
		) {
			Text(
				"Version: ${BuildConfig.VERSION_NAME}",
				style = MaterialTheme.typography.bodyMedium,
				color = MaterialTheme.colorScheme.secondary,
				modifier = Modifier.clickable {
					if (BuildConfig.DEBUG || BuildConfig.BUILD_TYPE == "prerelease") {
						navController.go(Destination.Environment.route)
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
