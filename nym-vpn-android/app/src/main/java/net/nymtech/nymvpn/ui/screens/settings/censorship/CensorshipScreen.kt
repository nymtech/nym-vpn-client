package net.nymtech.nymvpn.ui.screens.settings.censorship

import android.content.res.Configuration
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.AppUiState
import net.nymtech.nymvpn.ui.Route
import net.nymtech.nymvpn.ui.common.navigation.LocalNavController
import net.nymtech.nymvpn.ui.screens.settings.censorship.components.ConnectionStatus
import net.nymtech.nymvpn.ui.screens.settings.censorship.components.QuicInfoModal
import net.nymtech.nymvpn.ui.screens.settings.censorship.components.QuicInfoModalData
import net.nymtech.nymvpn.ui.screens.settings.censorship.components.QuicSection
import net.nymtech.nymvpn.ui.screens.settings.censorship.components.StealthApiSection
import net.nymtech.nymvpn.ui.screens.settings.censorship.components.getConnectionStatus
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.ui.theme.Typography
import net.nymtech.nymvpn.util.extensions.scaledHeight
import net.nymtech.nymvpn.util.extensions.scaledWidth

@Composable
fun CensorshipScreen(appUiState: AppUiState, viewModel: CensorshipViewModel = hiltViewModel()) {
	val navController = LocalNavController.current
	val uiState by viewModel.uiState.collectAsStateWithLifecycle()
	var quicInfoModal by remember { mutableStateOf(false to ConnectionStatus.Disconnected) }
	val quicModalData by remember(quicInfoModal.second) { mutableStateOf(QuicInfoModalData.getQuicInfoModalData(quicInfoModal.second)) }

	CensorshipScreen(
		showQUICSection = uiState.showQUICSection,
		showDomainFrontingSection = uiState.showDomainSection,
		appUiState.settings.quicEnabled,
		onQuicEnable = {
			val connectionStatus = getConnectionStatus(appUiState)
			if (it && connectionStatus == ConnectionStatus.FastNonQuicConnected || !it && connectionStatus == ConnectionStatus.FastQuicConnected) {
				quicInfoModal = true to connectionStatus
			}
			viewModel.onQUICEnabled(it)
		},
	)

	fun dismissQuicInfoModal() {
		quicInfoModal = false to quicInfoModal.second
	}

	fun onPrimaryButtonClicked() {
		dismissQuicInfoModal()
		viewModel.disconnect()
		navController.navigate(route = Route.Main(autoStart = true)) {
			popUpTo(Route.Main()) {
				inclusive = true
			}
			launchSingleTop = true
		}
	}

	QuicInfoModal(quicInfoModal.first, quicModalData, { onPrimaryButtonClicked() }, { dismissQuicInfoModal() })
}

@Composable
fun CensorshipScreen(showQUICSection: Boolean, showDomainFrontingSection: Boolean, quicEnabled: Boolean, onQuicEnable: (enabled: Boolean) -> Unit) {
	Column(
		horizontalAlignment = Alignment.Start,
		verticalArrangement = Arrangement.spacedBy(24.dp.scaledHeight(), Alignment.Top),
		modifier = Modifier
			.fillMaxSize()
			.padding(horizontal = 24.dp.scaledWidth()),
	) {
		Text(
			text = stringResource(R.string.censorship_description),
			style = Typography.bodyMedium,
			color = MaterialTheme.colorScheme.outline,
			fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
			modifier = Modifier
				.fillMaxWidth()
				.padding(top = 16.dp),
		)
		if (showQUICSection) {
			QuicSection(quicEnabled, onQuicEnable)
		}
		if (showDomainFrontingSection) {
			StealthApiSection()
		}
	}
}

@Composable
@Preview(uiMode = Configuration.UI_MODE_NIGHT_YES)
internal fun PreviewCensorshipScreen() {
	NymVPNTheme(Theme.default()) {
		CensorshipScreen(showQUICSection = true, showDomainFrontingSection = true, true, onQuicEnable = {})
	}
}
