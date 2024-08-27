package net.nymtech.nymvpn.ui.screens.settings.environment

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import net.nymtech.nymvpn.ui.AppUiState
import net.nymtech.nymvpn.ui.common.buttons.IconSurfaceButton
import net.nymtech.nymvpn.util.extensions.scaledHeight
import net.nymtech.nymvpn.util.extensions.scaledWidth
import net.nymtech.vpn.Tunnel

@Composable
fun EnvironmentScreen(appUiState: AppUiState, viewModel: EnvironmentViewModel = hiltViewModel()) {
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
				viewModel.onEnvironmentChange(Tunnel.Environment.CANARY)
			},
			selected = appUiState.settings.environment == Tunnel.Environment.CANARY,
		)
		IconSurfaceButton(
			title = Tunnel.Environment.SANDBOX.name,
			onClick = {
				viewModel.onEnvironmentChange(Tunnel.Environment.SANDBOX)
			},
			selected = appUiState.settings.environment == Tunnel.Environment.SANDBOX,
		)
		IconSurfaceButton(
			title = Tunnel.Environment.MAINNET.name,
			onClick = {
				viewModel.onEnvironmentChange(Tunnel.Environment.MAINNET)
			},
			selected = appUiState.settings.environment == Tunnel.Environment.MAINNET,
		)
	}
}
