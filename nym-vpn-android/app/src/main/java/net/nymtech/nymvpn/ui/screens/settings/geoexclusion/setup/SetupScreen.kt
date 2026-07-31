package net.nymtech.nymvpn.ui.screens.settings.geoexclusion.setup

import android.content.ClipData
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
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalClipboard
import androidx.compose.ui.platform.toClipEntry
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.withStyle
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.AppUiState
import net.nymtech.nymvpn.ui.screens.settings.geoexclusion.setup.components.CopyCard
import net.nymtech.nymvpn.ui.screens.settings.geoexclusion.setup.components.StepCard
import net.nymtech.nymvpn.ui.theme.CustomTypography
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.util.extensions.scaledHeight
import net.nymtech.nymvpn.util.extensions.scaledWidth

@Composable
fun SetupScreen(appUiState: AppUiState) {
	val clipboard = LocalClipboard.current
	val scope = rememberCoroutineScope()
	val proxyAddress = "127.0.0.1:${appUiState.vpnConfig.geoExclusionPort}"

	SetupScreen(
		proxyAddress = proxyAddress,
		onCopyAddress = {
			scope.launch {
				clipboard.setClipEntry(ClipData.newPlainText(proxyAddress, proxyAddress).toClipEntry())
			}
		},
	)
}

@Composable
fun SetupScreen(proxyAddress: String, onCopyAddress: () -> Unit) {
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
		Text(
			text = stringResource(R.string.setup_instructions_per_app_title),
			style = CustomTypography.titleMediumBold,
			color = MaterialTheme.colorScheme.onPrimaryContainer,
			modifier = Modifier
				.fillMaxWidth()
				.padding(top = 18.dp),
		)
		Text(
			text = stringResource(R.string.setup_instructions_per_app_description),
			style = MaterialTheme.typography.bodyMedium,
			color = MaterialTheme.colorScheme.onBackground,
			modifier = Modifier
				.fillMaxWidth()
				.padding(top = 8.dp),
		)
		StepCard(1, AnnotatedString(stringResource(R.string.setup_instructions_pert_app_step)), modifier = Modifier.padding(top = 8.dp))

		Text(
			text = stringResource(R.string.setup_instructions_browser_title),
			style = CustomTypography.titleMediumBold,
			color = MaterialTheme.colorScheme.onPrimaryContainer,
			modifier = Modifier
				.fillMaxWidth()
				.padding(top = 18.dp),
		)
		Text(
			text = stringResource(R.string.setup_instructions_browser_description),
			style = MaterialTheme.typography.bodyMedium,
			color = MaterialTheme.colorScheme.onBackground,
			modifier = Modifier
				.fillMaxWidth()
				.padding(top = 8.dp),
		)
		val port = proxyAddress.substringAfterLast(":")
		val browserStepText = buildAnnotatedString {
			append(stringResource(R.string.setup_instructions_browser_step_1))
			append(" ")
			withStyle(SpanStyle(color = MaterialTheme.colorScheme.primary)) {
				append(stringResource(R.string.setup_instructions_browser_step_2))
			}
			append(stringResource(R.string.setup_instructions_browser_step_3))
			append(" ")
			withStyle(SpanStyle(color = MaterialTheme.colorScheme.primary)) {
				append(port)
			}
			append(stringResource(R.string.setup_instructions_browser_step_5))
		}
		StepCard(1, browserStepText, modifier = Modifier.padding(top = 8.dp))

		Text(
			text = stringResource(R.string.setup_instructions_wallet_title),
			style = CustomTypography.titleMediumBold,
			color = MaterialTheme.colorScheme.onPrimaryContainer,
			modifier = Modifier
				.fillMaxWidth()
				.padding(top = 18.dp),
		)
		val walletStepText = buildAnnotatedString {
			append(stringResource(R.string.setup_instructions_wallet_step_1))
			append("\n")
			withStyle(SpanStyle(color = MaterialTheme.colorScheme.primary)) {
				append(stringResource(R.string.setup_instructions_wallet_step_2))
			}
			append(".")
		}
		StepCard(1, walletStepText, modifier = Modifier.padding(top = 8.dp))

		CopyCard(
			title = stringResource(R.string.setup_instructions_proxy_title),
			value = proxyAddress,
			onClick = onCopyAddress,
			modifier = Modifier.padding(top = 8.dp),
		)
	}
}

@Composable
@Preview(uiMode = Configuration.UI_MODE_NIGHT_YES)
internal fun PreviewSetupScreen() {
	NymVPNTheme(Theme.default()) {
		SetupScreen(
			proxyAddress = "127.0.0.1:1081",
			onCopyAddress = {},
		)
	}
}
