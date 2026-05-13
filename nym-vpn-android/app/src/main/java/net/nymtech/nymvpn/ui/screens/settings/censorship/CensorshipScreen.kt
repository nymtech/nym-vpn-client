package net.nymtech.nymvpn.ui.screens.settings.censorship

import android.content.res.Configuration
import android.widget.Toast
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import kotlinx.coroutines.flow.collectLatest
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.AppUiState
import net.nymtech.nymvpn.ui.common.events.UiEvent
import net.nymtech.nymvpn.ui.screens.settings.censorship.components.AmneziaSection
import net.nymtech.nymvpn.ui.screens.settings.censorship.components.QuicSection
import net.nymtech.nymvpn.ui.screens.settings.censorship.components.StealthApiSection
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.util.extensions.scaledWidth

@Composable
fun CensorshipScreen(appUiState: AppUiState, viewModel: CensorshipViewModel = hiltViewModel()) {
	val context = LocalContext.current

	LaunchedEffect(viewModel) {
		viewModel.events.collectLatest { event ->
			when (event) {
				UiEvent.ReconnectStarted -> {
					Toast.makeText(context, context.getString(R.string.censorship_event_reconnecting), Toast.LENGTH_SHORT).show()
				}
			}
		}
	}

	CensorshipScreen(
		quicEnabled = appUiState.settings.quicEnabled,
		onQuicEnable = { enabled -> viewModel.onQUICEnabled(enabled) },
		stealthModeEnabled = appUiState.vpnConfig.stealthMode,
		onStealthModeEnable = { enabled -> viewModel.onStealthModeEnabled(enabled) },
	)
}

@Composable
fun CensorshipScreen(quicEnabled: Boolean, onQuicEnable: (enabled: Boolean) -> Unit, stealthModeEnabled: Boolean = false, onStealthModeEnable: (Boolean) -> Unit = {}) {
	val scrollState = rememberScrollState()
	Column(
		horizontalAlignment = Alignment.Start,
		modifier = Modifier
			.fillMaxSize()
			.verticalScroll(scrollState)
			.padding(horizontal = 12.dp.scaledWidth())
			.navigationBarsPadding(),
	) {
		Text(
			text = stringResource(R.string.censorship_description),
			style = MaterialTheme.typography.bodyMedium,
			color = MaterialTheme.colorScheme.onBackground,
			fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
			modifier = Modifier
				.fillMaxWidth()
				.padding(top = 16.dp),
		)
		QuicSection(
			quicEnabled,
			onQuicEnable,
			shape = RoundedCornerShape(8.dp, 8.dp, 0.dp, 0.dp),
			modifier = Modifier
				.fillMaxWidth()
				.padding(top = 24.dp),
		)
		AmneziaSection(
			shape = RoundedCornerShape(0.dp, 0.dp, 8.dp, 8.dp),
			modifier = Modifier
				.fillMaxWidth()
				.padding(top = 1.dp),
		)
		StealthApiSection(
			isEnabled = stealthModeEnabled,
			onEnable = onStealthModeEnable,
			modifier = Modifier.fillMaxWidth().padding(top = 24.dp),
		)
	}
}

@Composable
@Preview(uiMode = Configuration.UI_MODE_NIGHT_YES)
internal fun PreviewCensorshipScreen() {
	NymVPNTheme(Theme.default()) {
		CensorshipScreen(quicEnabled = true, onQuicEnable = {})
	}
}
