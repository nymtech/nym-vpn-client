package net.nymtech.nymvpn.ui.screens.settings.tuning

import android.content.res.Configuration
import android.widget.Toast
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
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.outlined.OpenInNew
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.collectAsState
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextDecoration
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.hilt.lifecycle.viewmodel.compose.hiltViewModel
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.AppUiState
import net.nymtech.nymvpn.ui.common.buttons.MainStyledButton
import net.nymtech.nymvpn.ui.common.buttons.OutlineStyledButton
import net.nymtech.nymvpn.ui.screens.settings.tuning.components.BackgroundCoverTrafficSection
import net.nymtech.nymvpn.ui.screens.settings.tuning.components.MixingDelaysSection
import net.nymtech.nymvpn.ui.screens.settings.tuning.components.PerformanceSection
import net.nymtech.nymvpn.ui.screens.settings.tuning.components.SendTrafficSection
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.util.extensions.openWebUrl
import net.nymtech.nymvpn.util.extensions.scaledHeight
import net.nymtech.nymvpn.util.extensions.scaledWidth
import nym_vpn_lib_types.BackgroundCoverTrafficRate
import nym_vpn_lib_types.ContinuousTrafficSendingRate
import java.util.Locale

@Composable
fun MixnetTuningScreen(appUiState: AppUiState, viewModel: MixnetTuningViewModel = hiltViewModel()) {
	val uiState by viewModel.uiState.collectAsState()
	val context = LocalContext.current

	LaunchedEffect(uiState.validationError) {
		uiState.validationError?.let { error ->
			Toast.makeText(context, error, Toast.LENGTH_LONG).show()
			viewModel.clearError()
		}
	}

	// "1 Mbps", not "1.0 Mbps", to match the slider labels and the other apps
	val mbps = uiState.calculatedSpeedMbps
	val mbpsText = if (mbps % 1f == 0f) mbps.toInt().toString() else String.format(Locale.US, "%.1f", mbps)
	val speedStr = if (uiState.continuousTrafficEnabled) {
		stringResource(R.string.mixnet_tuning_speed_up_to, mbpsText)
	} else {
		stringResource(R.string.mixnet_tuning_internet_speed)
	}

	// Rounding ((val + 9) / 10) * 10
	val roundedLatency = ((uiState.calculatedLatencyMs + 9) / 10).toInt() * 10
	val latencyStr = stringResource(R.string.mixnet_tuning_latency_at_least, roundedLatency)

	MixnetTuningScreen(
		speed = speedStr,
		latency = latencyStr,

		continuousTrafficEnabled = uiState.continuousTrafficEnabled,
		onContinuousTrafficEnable = viewModel::onContinuousTrafficEnable,
		continuousTrafficRate = uiState.continuousTrafficRate,
		onContinuousTrafficRateChange = viewModel::onContinuousTrafficRateChange,

		backgroundCoverEnabled = uiState.backgroundCoverEnabled,
		onBackgroundCoverEnable = viewModel::onBackgroundCoverEnable,
		backgroundCoverRate = uiState.backgroundCoverRate,
		onBackgroundCoverRateChange = viewModel::onBackgroundCoverRateChange,

		delayValue = uiState.averagePacketDelay,
		delayValueRange = uiState.mixingDelayRange,
		delayDefaultValue = uiState.mixingDelayDefault,
		onDelayValueChange = viewModel::onDelayValueChange,

		saveButtonEnabled = uiState.hasUnsavedChanges,
		restoreButtonEnabled = !uiState.isCurrentStateDefault,

		onSaveSettingsClick = viewModel::onSaveSettingsClick,
		onRestoreDefaultClick = viewModel::onRestoreDefaultClick,
	)
}

@Composable
fun MixnetTuningScreen(
	speed: String,
	latency: String,
	continuousTrafficEnabled: Boolean,
	onContinuousTrafficEnable: (enabled: Boolean) -> Unit,
	continuousTrafficRate: ContinuousTrafficSendingRate,
	onContinuousTrafficRateChange: (ContinuousTrafficSendingRate) -> Unit,
	backgroundCoverEnabled: Boolean,
	onBackgroundCoverEnable: (enabled: Boolean) -> Unit,
	backgroundCoverRate: BackgroundCoverTrafficRate,
	onBackgroundCoverRateChange: (BackgroundCoverTrafficRate) -> Unit,
	delayValue: Float,
	delayValueRange: ClosedFloatingPointRange<Float>,
	delayDefaultValue: Float,
	onDelayValueChange: (Float) -> Unit,
	saveButtonEnabled: Boolean,
	restoreButtonEnabled: Boolean,
	onSaveSettingsClick: () -> Unit,
	onRestoreDefaultClick: () -> Unit,
) {
	val scrollState = rememberScrollState()
	val interactionSource = remember { MutableInteractionSource() }
	val context = LocalContext.current
	val url = stringResource(R.string.mixnet_tuning_link)

	Column(
		horizontalAlignment = Alignment.Start,
		verticalArrangement = Arrangement.spacedBy(16.dp),
		modifier = Modifier
			.fillMaxSize()
			.verticalScroll(scrollState)
			.padding(horizontal = 16.dp.scaledWidth(), vertical = 16.dp.scaledHeight())
			.navigationBarsPadding(),
	) {
		Text(
			text = stringResource(R.string.mixnet_tuning_description),
			style = MaterialTheme.typography.bodyMedium,
			color = MaterialTheme.colorScheme.onBackground,
		)

		PerformanceSection(speed = speed, latency = latency)

		SendTrafficSection(
			trafficEnabled = continuousTrafficEnabled,
			onTrafficEnable = onContinuousTrafficEnable,
			trafficRate = continuousTrafficRate,
			onTrafficRateChange = onContinuousTrafficRateChange,
		)

		BackgroundCoverTrafficSection(
			enabled = backgroundCoverEnabled,
			onEnable = onBackgroundCoverEnable,
			rate = backgroundCoverRate,
			onRateChange = onBackgroundCoverRateChange,
		)

		MixingDelaysSection(
			delayValue = delayValue,
			valueRange = delayValueRange,
			defaultValue = delayDefaultValue,
			onDelayValueChange = onDelayValueChange,
		)

		Row(
			verticalAlignment = Alignment.CenterVertically,
			modifier = Modifier.clickable(interactionSource = interactionSource, indication = null) {
				context.openWebUrl(url)
			},
		) {
			Text(
				text = stringResource(R.string.mixnet_tuning_link_text),
				style = MaterialTheme.typography.bodyMedium.copy(textDecoration = TextDecoration.Underline),
				color = MaterialTheme.colorScheme.onPrimaryContainer,
			)
			Spacer(modifier = Modifier.width(4.dp))
			Icon(
				imageVector = Icons.AutoMirrored.Outlined.OpenInNew,
				contentDescription = null,
				tint = MaterialTheme.colorScheme.onPrimaryContainer,
				modifier = Modifier.size(12.dp),
			)
		}

		MainStyledButton(
			onClick = onSaveSettingsClick,
			enabled = saveButtonEnabled,
			content = {
				Text(
					text = stringResource(R.string.mixnet_tuning_save_button),
					style = MaterialTheme.typography.titleMedium,
				)
			},
			modifier = Modifier.fillMaxWidth().height(48.dp.scaledHeight()),
			shape = RoundedCornerShape(12.dp),
		)

		// Always shown, greyed out while everything is at its default (as on Apple and desktop)
		OutlineStyledButton(
			onClick = onRestoreDefaultClick,
			enabled = restoreButtonEnabled,
			content = {
				Text(
					text = stringResource(R.string.mixnet_tuning_restore_button),
					style = MaterialTheme.typography.titleMedium,
				)
			},
			borderColor = MaterialTheme.colorScheme.onBackground.copy(alpha = if (restoreButtonEnabled) 1f else 0.38f),
			modifier = Modifier
				.fillMaxWidth()
				.height(48.dp.scaledHeight()),
			shape = RoundedCornerShape(12.dp),
		)
	}
}

@Composable
@Preview(uiMode = Configuration.UI_MODE_NIGHT_YES)
internal fun PreviewMixnetTuningScreen() {
	NymVPNTheme(Theme.default()) {
		MixnetTuningScreen(
			speed = "Up to 1 Mbps",
			latency = "At least 690 ms",
			continuousTrafficEnabled = true,
			onContinuousTrafficEnable = {},
			continuousTrafficRate = ContinuousTrafficSendingRate.MS20,
			onContinuousTrafficRateChange = {},
			backgroundCoverEnabled = false,
			onBackgroundCoverEnable = {},
			backgroundCoverRate = BackgroundCoverTrafficRate.MS200,
			onBackgroundCoverRateChange = {},
			delayValue = 15f,
			delayValueRange = 0f..200f,
			delayDefaultValue = 15f,
			onDelayValueChange = {},
			saveButtonEnabled = false,
			restoreButtonEnabled = true,
			onSaveSettingsClick = {},
			onRestoreDefaultClick = {},
		)
	}
}
