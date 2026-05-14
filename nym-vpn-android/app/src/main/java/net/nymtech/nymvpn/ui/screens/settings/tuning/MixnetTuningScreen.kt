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
import net.nymtech.nymvpn.util.extensions.openWebUrl
import net.nymtech.nymvpn.util.extensions.scaledHeight
import net.nymtech.nymvpn.util.extensions.scaledWidth
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

	val internetSpeedString = stringResource(R.string.mixnet_tuning_internet_speed)

	val speedStr = remember(uiState.trafficEnabled, uiState.calculatedSpeedMbps) {
		if (!uiState.trafficEnabled) {
			internetSpeedString
		} else {
			if (uiState.calculatedSpeedMbps < 0.1f && uiState.calculatedSpeedMbps > 0f) {
				"< 0.1 Mbps"
			} else {
				String.format(Locale.US, "Up to %.1f Mbps", uiState.calculatedSpeedMbps)
			}
		}
	}

	val latencyStr = remember(uiState.calculatedLatencyMs) {
		// Rounding ((val + 9) / 10) * 10
		val roundedLatency = ((uiState.calculatedLatencyMs + 9) / 10).toInt() * 10
		"At least $roundedLatency ms"
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
	val url = stringResource(R.string.mixnet_tuning_link)

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
			style = MaterialTheme.typography.bodyLarge,
			color = MaterialTheme.colorScheme.onPrimaryContainer,
			textAlign = TextAlign.Center,
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
