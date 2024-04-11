package net.nymtech.nymvpn.ui.screens.settings

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.AccountCircle
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Switch
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.res.vectorResource
import androidx.compose.ui.text.buildAnnotatedString
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
import net.nymtech.nymvpn.ui.common.buttons.surface.SelectionItem
import net.nymtech.nymvpn.ui.common.buttons.surface.SurfaceSelectionGroupButton
import net.nymtech.nymvpn.util.scaledHeight
import net.nymtech.nymvpn.util.scaledWidth

@Composable
fun SettingsScreen(
	navController: NavController,
	appUiState: AppUiState,
	appViewModel: AppViewModel,
	viewModel: SettingsViewModel = hiltViewModel(),
) {
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
		if (!appUiState.loggedIn) {
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
		} else {
			// TODO get real account numbers, mock for now
			val accountDescription =
				buildAnnotatedString {
					append("31")
					append(" ")
					append(stringResource(id = R.string.of))
					append(" ")
					append("31")
					append(" ")
					append(stringResource(id = R.string.days_left))
				}
			SurfaceSelectionGroupButton(
				listOf(
					SelectionItem(
						Icons.Filled.AccountCircle,
						onClick = { navController.navigate(NavItem.Settings.Account.route) },
						title = stringResource(R.string.credential),
						description = accountDescription.text,
					),
				),
			)
		}
		SurfaceSelectionGroupButton(
			listOf(
				SelectionItem(
					ImageVector.vectorResource(R.drawable.auto),
					{
						Switch(
							uiState.isAutoConnectEnabled,
							{ viewModel.onAutoConnectSelected(it) },
							modifier =
							Modifier
								.height(32.dp.scaledHeight())
								.width(52.dp.scaledWidth()),
						)
					},
					stringResource(R.string.auto_connect),
					stringResource(R.string.auto_connect_description),
					{},
				),
				SelectionItem(
					ImageVector.vectorResource(R.drawable.two),
					{
						Switch(
							uiState.isFirstHopSelectionEnabled,
							{ appViewModel.onEntryLocationSelected(it) },
							modifier =
							Modifier
								.height(32.dp.scaledHeight())
								.width(52.dp.scaledWidth()),
						)
					},
					stringResource(R.string.entry_location),
					stringResource(R.string.entry_location_description),
					{},
				),
			),
		)
		SurfaceSelectionGroupButton(
			listOf(
				SelectionItem(
					ImageVector.vectorResource(R.drawable.contrast),
					title = stringResource(R.string.display_theme),
					onClick = { navController.navigate(NavItem.Settings.Display.route) },
				),
			),
		)
		SurfaceSelectionGroupButton(
			listOf(
				SelectionItem(
					ImageVector.vectorResource(R.drawable.logs),
					title = stringResource(R.string.logs),
					onClick = { navController.navigate(NavItem.Settings.Logs.route) },
				),
			),
		)
		SurfaceSelectionGroupButton(
			listOf(
				SelectionItem(
					ImageVector.vectorResource(R.drawable.feedback),
					title = stringResource(R.string.feedback),
					onClick = { navController.navigate(NavItem.Settings.Feedback.route) },
				),
				SelectionItem(
					ImageVector.vectorResource(R.drawable.support),
					title = stringResource(R.string.support),
					// TODO fix, direct to support
					onClick = { navController.navigate(NavItem.Settings.Support.route) },
				),
			),
		)
		SurfaceSelectionGroupButton(
			listOf(
				SelectionItem(
					title = stringResource(R.string.legal),
					onClick = { navController.navigate(NavItem.Settings.Legal.route) },
				),
			),
		)
		if (appUiState.loggedIn) {
			SurfaceSelectionGroupButton(
				listOf(
					SelectionItem(
						title = stringResource(R.string.log_out),
						onClick = {
							navController.navigate(NavItem.Main.route)
							viewModel.onLogOutSelected()
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
				"Version: ${BuildConfig.VERSION_NAME}",
				style = MaterialTheme.typography.bodyMedium,
				color = MaterialTheme.colorScheme.secondary,
			)
		}
	}
}
