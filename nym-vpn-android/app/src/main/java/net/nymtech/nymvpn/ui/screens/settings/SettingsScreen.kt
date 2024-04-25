package net.nymtech.nymvpn.ui.screens.settings

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.text.ClickableText
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.BugReport
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.res.vectorResource
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.withStyle
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import androidx.navigation.NavController
import net.nymtech.nymvpn.BuildConfig
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.AppUiState
import net.nymtech.nymvpn.ui.AppViewModel
import net.nymtech.nymvpn.ui.NavItem
import net.nymtech.nymvpn.ui.common.buttons.MainStyledButton
import net.nymtech.nymvpn.ui.common.buttons.ScaledSwitch
import net.nymtech.nymvpn.ui.common.buttons.surface.SelectionItem
import net.nymtech.nymvpn.ui.common.buttons.surface.SurfaceSelectionGroupButton
import net.nymtech.nymvpn.util.scaledHeight
import net.nymtech.nymvpn.util.scaledWidth
import net.nymtech.vpn.model.VpnState

@Composable
fun SettingsScreen(
	navController: NavController,
	appViewModel: AppViewModel,
	appUiState: AppUiState,
	viewModel: SettingsViewModel = hiltViewModel(),
) {
	val context = LocalContext.current

	val uiState by viewModel.uiState.collectAsStateWithLifecycle()

	Column(
		horizontalAlignment = Alignment.Start,
		verticalArrangement = Arrangement.spacedBy(24.dp, Alignment.Top),
		modifier =
		Modifier
			.verticalScroll(rememberScrollState())
			.fillMaxSize()
			.padding(top = 24.dp)
			.padding(horizontal = 24.dp.scaledWidth()),
	) {
// 		if (!appUiState.loggedIn) {
		MainStyledButton(
			onClick = { navController.navigate(NavItem.Settings.Login.route) },
			content = {
				Text(
					stringResource(id = R.string.add_cred_to_connect),
					style = MaterialTheme.typography.labelLarge,
				)
			},
			color = MaterialTheme.colorScheme.primary,
		)
// 		} else {
		// TODO disable account for now
// 			val accountDescription =
// 				buildAnnotatedString {
// 					append("31")
// 					append(" ")
// 					append(stringResource(id = R.string.of))
// 					append(" ")
// 					append("31")
// 					append(" ")
// 					append(stringResource(id = R.string.days_left))
// 				}
// 			SurfaceSelectionGroupButton(
// 				listOf(
// 					SelectionItem(
// 						Icons.Filled.AccountCircle,
// 						onClick = { navController.navigate(NavItem.Settings.Account.route) },
// 						title = { Text(stringResource(R.string.credential), style = MaterialTheme.typography.bodyLarge.copy(MaterialTheme.colorScheme.onSurface))},
// 						description = { Text(accountDescription.text, style = MaterialTheme.typography.bodyMedium.copy(MaterialTheme.colorScheme.outline))},
// 					),
// 				),
// 			)
// 		}
		SurfaceSelectionGroupButton(
			listOf(
				SelectionItem(
					ImageVector.vectorResource(R.drawable.auto),
					{
						ScaledSwitch(
							uiState.isAutoConnectEnabled,
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
					{},
				),
				SelectionItem(
					ImageVector.vectorResource(R.drawable.contrast),
					title = { Text(stringResource(R.string.display_theme), style = MaterialTheme.typography.bodyLarge.copy(MaterialTheme.colorScheme.onSurface)) },
					onClick = { navController.navigate(NavItem.Settings.Display.route) },
				),
			),
		)
		SurfaceSelectionGroupButton(
			listOf(
				SelectionItem(
					ImageVector.vectorResource(R.drawable.two),
					{
						ScaledSwitch(
							uiState.isFirstHopSelectionEnabled,
							onClick = { appViewModel.onEntryLocationSelected(it) },
							modifier =
							Modifier
								.height(32.dp.scaledHeight())
								.width(52.dp.scaledWidth()),
							enabled = (appUiState.vpnState is VpnState.Down),
						)
					},
					title = { Text(stringResource(R.string.entry_location), style = MaterialTheme.typography.bodyLarge.copy(MaterialTheme.colorScheme.onSurface)) },
					description = {
						Text(
							stringResource(id = R.string.entry_location_description),
							style = MaterialTheme.typography.bodyMedium.copy(MaterialTheme.colorScheme.outline),
						)
					},
					{},
				),
				SelectionItem(
					ImageVector.vectorResource(R.drawable.logs),
					title = { Text(stringResource(R.string.logs), style = MaterialTheme.typography.bodyLarge.copy(MaterialTheme.colorScheme.onSurface)) },
					onClick = { navController.navigate(NavItem.Settings.Logs.route) },
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
					onClick = { navController.navigate(NavItem.Settings.Feedback.route) },
				),
				SelectionItem(
					ImageVector.vectorResource(R.drawable.support),
					title = { Text(stringResource(R.string.support), style = MaterialTheme.typography.bodyLarge.copy(MaterialTheme.colorScheme.onSurface)) },
					onClick = { navController.navigate(NavItem.Settings.Support.route) },
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
								appViewModel.openWebPage(annotation.item, context)
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
					onClick = { navController.navigate(NavItem.Settings.Legal.route) },
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
			)
		}
	}
}
