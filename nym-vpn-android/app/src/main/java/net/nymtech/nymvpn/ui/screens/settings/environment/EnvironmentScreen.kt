package net.nymtech.nymvpn.ui.screens.settings.environment

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.AppUiState
import net.nymtech.nymvpn.ui.AppViewModel
import net.nymtech.nymvpn.ui.common.buttons.IconSurfaceButton
import net.nymtech.nymvpn.ui.common.navigation.LocalNavController
import net.nymtech.nymvpn.ui.common.navigation.NavBarState
import net.nymtech.nymvpn.ui.common.navigation.NavIcon
import net.nymtech.nymvpn.ui.common.navigation.NavTitle
import net.nymtech.nymvpn.util.extensions.scaledHeight
import net.nymtech.nymvpn.util.extensions.scaledWidth
import net.nymtech.vpn.backend.Tunnel

@Composable
fun EnvironmentScreen(appUiState: AppUiState, appViewModel: AppViewModel, viewModel: EnvironmentViewModel = hiltViewModel()) {
	val navController = LocalNavController.current

	LaunchedEffect(Unit) {
		appViewModel.onNavBarStateChange(
			NavBarState(
				title = { NavTitle(stringResource(R.string.environment)) },
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
		verticalArrangement = Arrangement.spacedBy(24.dp.scaledHeight(), Alignment.Top),
		modifier =
		Modifier
			.fillMaxSize()
			.padding(top = 24.dp.scaledHeight())
			.padding(horizontal = 24.dp.scaledWidth()),
	) {
		IconSurfaceButton(
			title = Tunnel.Environment.CANARY.name,
			onClick = {
				if (appUiState.settings.environment == Tunnel.Environment.CANARY) return@IconSurfaceButton
				viewModel.onEnvironmentChange(Tunnel.Environment.CANARY)
			},
			selected = appUiState.settings.environment == Tunnel.Environment.CANARY,
		)
		IconSurfaceButton(
			title = Tunnel.Environment.SANDBOX.name,
			onClick = {
				if (appUiState.settings.environment == Tunnel.Environment.SANDBOX) return@IconSurfaceButton
				viewModel.onEnvironmentChange(Tunnel.Environment.SANDBOX)
			},
			selected = appUiState.settings.environment == Tunnel.Environment.SANDBOX,
		)
		IconSurfaceButton(
			title = Tunnel.Environment.MAINNET.name,
			onClick = {
				if (appUiState.settings.environment == Tunnel.Environment.MAINNET) return@IconSurfaceButton
				viewModel.onEnvironmentChange(Tunnel.Environment.MAINNET)
			},
			selected = appUiState.settings.environment == Tunnel.Environment.MAINNET,
		)
	}
}
