package net.nymtech.nymvpn.ui.screens.settings.tuning

import android.content.res.Configuration
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.outlined.OpenInNew
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextDecoration
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.AppUiState
import net.nymtech.nymvpn.ui.common.buttons.MainStyledButton
import net.nymtech.nymvpn.ui.common.buttons.OutlineStyledButton
import net.nymtech.nymvpn.ui.screens.settings.tuning.components.MixingDelaysSection
import net.nymtech.nymvpn.ui.screens.settings.tuning.components.PerformanceSection
import net.nymtech.nymvpn.ui.screens.settings.tuning.components.SendTrafficSection
import net.nymtech.nymvpn.ui.theme.CustomTypography
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.ui.theme.Typography
import net.nymtech.nymvpn.util.extensions.openWebUrl
import net.nymtech.nymvpn.util.extensions.scaledHeight
import net.nymtech.nymvpn.util.extensions.scaledWidth
import java.util.Locale

@Composable
fun MixnetTuningScreen(appUiState: AppUiState, viewModel: MixnetTuningViewModel = hiltViewModel()) {
	val uiState by viewModel.uiState.collectAsState()

	val internetSpeedString = stringResource(R.string.mixnet_tuning_internet_speed)

	val (speedStr, latencyStr) = remember(uiState.currentTrafficValue, uiState.averagePacketDelay, uiState.trafficEnabled) {
		calculatePerformanceMetrics(
			trafficValueMs = uiState.currentTrafficValue,
			delayValueMs = uiState.averagePacketDelay,
			trafficEnabled = uiState.trafficEnabled,
			internetSpeedLabel = internetSpeedString,
		)
	}

	MixnetTuningScreen(
		speed = speedStr,
		latency = latencyStr,

		trafficEnabled = uiState.trafficEnabled,
		onTrafficEnable = viewModel::onTrafficEnable,

		trafficValue = uiState.currentTrafficValue,
		onTrafficValueChange = viewModel::onTrafficValueChange,

		delayValue = uiState.averagePacketDelay,
		onDelayValueChange = viewModel::onDelayValueChange,

		saveButtonEnabled = uiState.hasUnsavedChanges,
		showRestoreButton = !uiState.isCurrentStateDefault,

		onSaveSettingsClick = viewModel::onSaveSettingsClick,
		onRestoreDefaultClick = viewModel::onRestoreDefaultClick,
	)
}

@Composable
fun MixnetTuningScreen(
	speed: String,
	latency: String,
	trafficEnabled: Boolean,
	onTrafficEnable: (enabled: Boolean) -> Unit,
	trafficValue: Float,
	onTrafficValueChange: (Float) -> Unit,
	delayValue: Float,
	onDelayValueChange: (Float) -> Unit,
	saveButtonEnabled: Boolean,
	showRestoreButton: Boolean,
	onSaveSettingsClick: () -> Unit,
	onRestoreDefaultClick: () -> Unit,
) {
	val scrollState = rememberScrollState()
	val interactionSource = remember { MutableInteractionSource() }
	val context = LocalContext.current

	Column(
		horizontalAlignment = Alignment.Start,
		verticalArrangement = Arrangement.spacedBy(24.dp),
		modifier = Modifier
			.fillMaxSize()
			.verticalScroll(scrollState)
			.padding(horizontal = 16.dp.scaledWidth(), vertical = 24.dp.scaledHeight())
			.navigationBarsPadding(),
	) {
		Text(
			text = stringResource(R.string.mixnet_tuning_description),
			style = Typography.bodyMedium,
			color = MaterialTheme.colorScheme.outline,
			textAlign = TextAlign.Center,
			fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
			modifier = Modifier
				.fillMaxWidth()
				.padding(top = 16.dp),
		)

		PerformanceSection(speed = speed, latency = latency)

		SendTrafficSection(
			trafficEnabled = trafficEnabled,
			onTrafficEnable = onTrafficEnable,
			trafficValue = trafficValue,
			onTrafficValueChange = onTrafficValueChange,
		)

		MixingDelaysSection(
			delayValue = delayValue,
			onDelayValueChange = onDelayValueChange,
		)

		Row(
			verticalAlignment = Alignment.CenterVertically,
			modifier = Modifier.clickable(interactionSource = interactionSource, indication = null) {
				context.openWebUrl(context.getString(R.string.mixnet_tuning_link))
			},
		) {
			Text(
				text = stringResource(R.string.mixnet_tuning_link_text),
				style = MaterialTheme.typography.bodyMedium.copy(textDecoration = TextDecoration.Underline),
				color = MaterialTheme.colorScheme.onBackground,
				fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
			)
			Spacer(modifier = Modifier.width(4.dp))
			Icon(
				imageVector = Icons.AutoMirrored.Outlined.OpenInNew,
				contentDescription = null,
				tint = MaterialTheme.colorScheme.onBackground,
				modifier = Modifier.size(12.dp),
			)
		}

		MainStyledButton(
			onClick = onSaveSettingsClick,
			enabled = saveButtonEnabled,
			content = {
				Text(
					text = stringResource(R.string.mixnet_tuning_save_button),
					style = CustomTypography.buttonMain,
				)
			},
			modifier = Modifier
				.fillMaxWidth()
				.height(42.dp.scaledHeight()),
		)

		if (showRestoreButton) {
			OutlineStyledButton(
				onClick = onRestoreDefaultClick,
				content = {
					Text(
						text = stringResource(R.string.mixnet_tuning_restore_button),
						style = CustomTypography.buttonMain,
					)
				},
				borderColor = MaterialTheme.colorScheme.onBackground,
				modifier = Modifier
					.fillMaxWidth()
					.height(42.dp.scaledHeight()),
			)
		}
	}
}

private fun calculatePerformanceMetrics(trafficValueMs: Float, delayValueMs: Float, trafficEnabled: Boolean, internetSpeedLabel: String): Pair<String, String> {
	val speedStr = if (!trafficEnabled) {
		internetSpeedLabel
	} else {
		val mbps = if (trafficValueMs > 0) {
			20f / trafficValueMs
		} else {
			0f
		}

		if (mbps < 0.1f && mbps > 0f) {
			"< 0.1 Mbps"
		} else {
			String.format(Locale.US, "Up to %.1f Mbps", mbps)
		}
	}

	val baseLatencyOneWay = 300
	val variableLatencyOneWay = (3 * delayValueMs).toInt()

	val rttLatency = 2 * (baseLatencyOneWay + variableLatencyOneWay)
	val roundedLatency = ((rttLatency + 9) / 10) * 10

	val latencyStr = "At least $roundedLatency ms"

	return speedStr to latencyStr
}

@Composable
@Preview(uiMode = Configuration.UI_MODE_NIGHT_YES)
internal fun PreviewMixnetTuningScreen() {
	NymVPNTheme(Theme.default()) {
		MixnetTuningScreen(
			speed = "Up to 1.0 Mbps",
			latency = "At least 690 ms",
			trafficEnabled = true,
			onTrafficEnable = {},
			trafficValue = 20f,
			onTrafficValueChange = {},
			delayValue = 15f,
			onDelayValueChange = {},
			saveButtonEnabled = false,
			showRestoreButton = true,
			onSaveSettingsClick = {},
			onRestoreDefaultClick = {},
		)
	}
}
