package net.nymtech.nymvpn.ui.screens.settings.dns

import android.content.res.Configuration
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import net.nymtech.nymvpn.ui.AppUiState
import net.nymtech.nymvpn.ui.common.navigation.LocalNavController
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.util.extensions.scaledWidth

@Composable
fun DnsScreen(appUiState: AppUiState, viewModel: DnsViewModel = hiltViewModel()) {
	val navController = LocalNavController.current

	DnsScreen(
	)
}

@Composable
fun DnsScreen() {
	val scrollState = rememberScrollState()
	Column(
		horizontalAlignment = Alignment.Start,
		modifier = Modifier
			.fillMaxSize()
			.verticalScroll(scrollState)
			.padding(horizontal = 24.dp.scaledWidth())
			.navigationBarsPadding(),
	) {
	}
}

@Composable
@Preview(uiMode = Configuration.UI_MODE_NIGHT_YES)
internal fun PreviewCensorshipScreen() {
	NymVPNTheme(Theme.default()) {
		DnsScreen()
	}
}
