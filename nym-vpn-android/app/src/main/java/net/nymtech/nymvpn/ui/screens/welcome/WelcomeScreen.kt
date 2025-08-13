package net.nymtech.nymvpn.ui.screens.welcome

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.asPaddingValues
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.imePadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.systemBars
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import net.nymtech.nymvpn.ui.AppUiState
import net.nymtech.nymvpn.ui.common.navigation.LocalNavController
import net.nymtech.nymvpn.ui.screens.welcome.components.ContinueButtonSection
import net.nymtech.nymvpn.ui.screens.welcome.components.SettingsSection
import net.nymtech.nymvpn.ui.screens.welcome.components.WelcomeSection
import net.nymtech.nymvpn.util.extensions.scaledHeight
import net.nymtech.nymvpn.util.extensions.scaledWidth

@Composable
fun WelcomeScreen(appUiState: AppUiState, viewModel: WelcomeViewModel = hiltViewModel()) {

	val padding = WindowInsets.systemBars.asPaddingValues()
	val navController = LocalNavController.current

	Column(
		horizontalAlignment = Alignment.CenterHorizontally,
		modifier = Modifier
			.fillMaxSize()
			.imePadding()
			.padding(horizontal = 24.dp.scaledWidth())
			.padding(padding),
	) {
		WelcomeSection(modifier = Modifier
			.padding(vertical = 24.dp.scaledHeight())
			.weight(1f))
		Column(
			horizontalAlignment = Alignment.CenterHorizontally,
			verticalArrangement = Arrangement.spacedBy(16.dp, Alignment.Bottom),
			modifier = Modifier.padding(vertical = 24.dp.scaledHeight()),
		) {
			SettingsSection(appUiState, viewModel)
			ContinueButtonSection(navController, viewModel)
		}
	}
}
