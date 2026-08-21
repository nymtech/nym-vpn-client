package net.nymtech.nymvpn.ui.screens.settings.geoexclusion.setup

import android.content.res.Configuration
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.withStyle
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.AppUiState
import net.nymtech.nymvpn.ui.screens.settings.geoexclusion.setup.components.StepsCard
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.util.extensions.scaledHeight
import net.nymtech.nymvpn.util.extensions.scaledWidth

@Composable
fun SetupScreen(appUiState: AppUiState) {
	SetupScreen(proxyAddress = "127.0.0.1:${appUiState.vpnConfig.geoExclusionPort}")
}

@Composable
fun SetupScreen(proxyAddress: String) {
	val port = proxyAddress.substringAfterLast(":")
	val socksStep = buildAnnotatedString {
		append(stringResource(R.string.setup_instructions_step_socks_host))
		append(" ")
		withStyle(SpanStyle(color = MaterialTheme.colorScheme.primary)) {
			append("127.0.0.1")
		}
		append(stringResource(R.string.setup_instructions_step_socks_port))
		append(" ")
		withStyle(SpanStyle(color = MaterialTheme.colorScheme.primary)) {
			append(port)
		}
		append(stringResource(R.string.setup_instructions_step_socks_suffix))
	}
	Column(
		horizontalAlignment = Alignment.Start,
		modifier = Modifier
			.fillMaxSize()
			.verticalScroll(rememberScrollState())
			.padding(horizontal = 20.dp.scaledWidth(), vertical = 20.dp.scaledHeight())
			.navigationBarsPadding(),
	) {
		Text(
			text = stringResource(R.string.setup_instructions_description),
			style = MaterialTheme.typography.bodyMedium,
			color = MaterialTheme.colorScheme.onBackground,
			modifier = Modifier.fillMaxWidth(),
		)
		StepsCard(
			steps = listOf(
				AnnotatedString(stringResource(R.string.setup_instructions_step_manual)),
				socksStep,
				AnnotatedString(stringResource(R.string.setup_instructions_step_dns)),
			),
			modifier = Modifier.padding(top = 12.dp),
		)
	}
}

@Composable
@Preview(uiMode = Configuration.UI_MODE_NIGHT_YES)
internal fun PreviewSetupScreen() {
	NymVPNTheme(Theme.default()) {
		SetupScreen(proxyAddress = "127.0.0.1:1081")
	}
}
