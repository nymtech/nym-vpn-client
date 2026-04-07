package net.nymtech.nymvpn.ui.screens.settings.developer

import android.content.res.Configuration
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
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.ui.AppUiState
import net.nymtech.nymvpn.ui.AppViewModel
import net.nymtech.nymvpn.ui.screens.settings.developer.components.ConnectionDataSection
import net.nymtech.nymvpn.ui.screens.settings.developer.components.DeveloperOptionsSection
import net.nymtech.nymvpn.ui.screens.settings.developer.components.MixnetStateSection
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.util.extensions.scaledHeight
import net.nymtech.nymvpn.util.extensions.scaledWidth
import net.nymtech.vpn.backend.Tunnel

@Composable
fun DeveloperScreen(appUiState: AppUiState, appViewModel: AppViewModel) {
	var environmentExpanded by remember { mutableStateOf(false) }
	var credentialExpanded by remember { mutableStateOf(false) }

	DeveloperScreen(
		appUiState = appUiState,
		environmentExpanded = environmentExpanded,
		onEnvironmentExpandedChange = { environmentExpanded = it },
		credentialExpanded = credentialExpanded,
		onCredentialExpandedChange = { credentialExpanded = it },
		onLogout = { appViewModel.logout() },
		onEnvironmentChange = { appViewModel.onEnvironmentChange(it) },
		onCredentialOverride = { appViewModel.onCredentialOverride(it) },
	)
}

@Composable
fun DeveloperScreen(
	appUiState: AppUiState,
	environmentExpanded: Boolean,
	onEnvironmentExpandedChange: (Boolean) -> Unit,
	credentialExpanded: Boolean,
	onCredentialExpandedChange: (Boolean) -> Unit,
	onLogout: suspend () -> Unit,
	onEnvironmentChange: suspend (Tunnel.Environment) -> Unit,
	onCredentialOverride: (Boolean?) -> Unit,
	padding: androidx.compose.foundation.layout.PaddingValues = WindowInsets.systemBars.asPaddingValues(),
) {
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
			environmentExpanded = environmentExpanded,
			onEnvironmentExpandedChange = onEnvironmentExpandedChange,
			credentialExpanded = credentialExpanded,
			onCredentialExpandedChange = onCredentialExpandedChange,
			onLogout = onLogout,
			onEnvironmentChange = onEnvironmentChange,
			onCredentialOverride = onCredentialOverride,
		)
	}
}

@Composable
@Preview(uiMode = Configuration.UI_MODE_NIGHT_YES)
internal fun PreviewDeveloperScreen() {
	NymVPNTheme(Theme.default()) {
		DeveloperScreen(
			appUiState = AppUiState(),
			environmentExpanded = false,
			onEnvironmentExpandedChange = {},
			credentialExpanded = false,
			onCredentialExpandedChange = {},
			onLogout = {},
			onEnvironmentChange = {},
			onCredentialOverride = {},
		)
	}
}
