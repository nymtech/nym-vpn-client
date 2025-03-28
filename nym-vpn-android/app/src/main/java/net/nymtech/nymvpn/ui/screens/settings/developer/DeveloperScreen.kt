package net.nymtech.nymvpn.ui.screens.settings.developer

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.asPaddingValues
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.systemBars
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.ui.AppUiState
import net.nymtech.nymvpn.ui.AppViewModel
import net.nymtech.nymvpn.util.extensions.scaledHeight
import net.nymtech.nymvpn.util.extensions.scaledWidth
import net.nymtech.nymvpn.ui.screens.settings.developer.components.ConnectionDataSection
import net.nymtech.nymvpn.ui.screens.settings.developer.components.DeveloperOptionsSection
import net.nymtech.nymvpn.ui.screens.settings.developer.components.MixnetStateSection

@Composable
fun DeveloperScreen(appUiState: AppUiState, appViewModel: AppViewModel) {
	val padding = WindowInsets.systemBars.asPaddingValues()

	var environmentExpanded by remember { mutableStateOf(false) }
	var credentialExpanded by remember { mutableStateOf(false) }

	Column(
		horizontalAlignment = Alignment.Start,
		verticalArrangement = Arrangement.spacedBy(24.dp.scaledHeight(), Alignment.Top),
		modifier = Modifier
			.fillMaxSize()
			.verticalScroll(rememberScrollState())
			.padding(top = 24.dp)
			.padding(horizontal = 24.dp.scaledWidth())
			.padding(bottom = padding.calculateBottomPadding()),
	) {
		ConnectionDataSection(appUiState = appUiState)
		MixnetStateSection(appUiState = appUiState)
		DeveloperOptionsSection(
			appUiState = appUiState,
			appViewModel = appViewModel,
			environmentExpanded = environmentExpanded,
			onEnvironmentExpandedChange = { environmentExpanded = it },
			credentialExpanded = credentialExpanded,
			onCredentialExpandedChange = { credentialExpanded = it },
		)
	}
}
