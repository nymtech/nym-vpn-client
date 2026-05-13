package net.nymtech.nymvpn.ui.screens.settings.tuning.components

import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Slider
import androidx.compose.material3.SliderDefaults
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.DpSize
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.data.domain.Settings
import net.nymtech.nymvpn.ui.theme.LocalNymColors
import kotlin.math.roundToInt

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun MixingDelaysSection(delayValue: Float, onDelayValueChange: (Float) -> Unit) {
	val interactionSource = remember { MutableInteractionSource() }

	val defaultDelay = Settings.MIXNET_CONFIG_DEFAULT.averagePacketDelay?.toInt()
	val currentInt = delayValue.roundToInt()
	val isDefault = currentInt == defaultDelay

	Card(
		shape = RoundedCornerShape(8.dp),
		colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.primaryContainer),
	) {
		Column(
			modifier = Modifier
				.fillMaxWidth()
				.padding(16.dp),
			verticalArrangement = Arrangement.spacedBy(16.dp),
		) {
			Text(
				text = stringResource(R.string.mixnet_tuning_delays_title),
				style = MaterialTheme.typography.titleMedium,
				color = MaterialTheme.colorScheme.onPrimaryContainer,
				maxLines = 2,
				overflow = TextOverflow.Ellipsis,
			)

			if (delayValue == 0f) {
				Text(
					text = stringResource(R.string.mixnet_tuning_traffic_warning),
					style = MaterialTheme.typography.bodySmall,
					color = LocalNymColors.current.warning,
					fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
				)
			} else {
				Text(
					text = stringResource(R.string.mixnet_tuning_delays_description),
					style = MaterialTheme.typography.bodySmall,
					color = MaterialTheme.colorScheme.onBackground,
					fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
				)
			}

			Row(
				modifier = Modifier.fillMaxWidth(),
				horizontalArrangement = Arrangement.SpaceBetween,
			) {
				Text(
					text = stringResource(R.string.mixnet_tuning_delays_min),
					style = MaterialTheme.typography.bodySmall,
					color = MaterialTheme.colorScheme.onBackground,
				)
				Text(
					text = stringResource(R.string.mixnet_tuning_max),
					style = MaterialTheme.typography.bodySmall,
					color = MaterialTheme.colorScheme.onBackground,
				)
			}

			Slider(
				value = delayValue,
				onValueChange = { newValue ->
					onDelayValueChange(newValue.roundToInt().toFloat())
				},
				valueRange = 0f..200f,
				interactionSource = interactionSource,
				modifier = Modifier.fillMaxWidth(),
				thumb = {
					SliderDefaults.Thumb(
						interactionSource = interactionSource,
						colors = SliderDefaults.colors(thumbColor = MaterialTheme.colorScheme.onBackground),
						thumbSize = DpSize(20.dp, 20.dp),
					)
				},
				track = { sliderState ->
					SliderDefaults.Track(
						sliderState = sliderState,
						modifier = Modifier.height(4.dp),
						colors = SliderDefaults.colors(
							activeTrackColor = MaterialTheme.colorScheme.primary,
							inactiveTrackColor = MaterialTheme.colorScheme.outline.copy(alpha = 0.2f),
						),
						thumbTrackGapSize = 0.dp,
						drawStopIndicator = null,
					)
				},
			)

			Row(
				modifier = Modifier.fillMaxWidth(),
				horizontalArrangement = Arrangement.SpaceBetween,
			) {
				Text(
					text = stringResource(R.string.mixnet_tuning_delays_low),
					style = MaterialTheme.typography.bodyMedium,
					color = MaterialTheme.colorScheme.onPrimaryContainer,
					modifier = Modifier.weight(1f),
					textAlign = TextAlign.Start,
				)

				Text(
					text = if (isDefault) {
						stringResource(R.string.mixnet_tuning_delays_default)
					} else {
						stringResource(R.string.mixnet_tuning_delays_current, currentInt.toString())
					},
					style = MaterialTheme.typography.bodyMedium,
					color = if (isDefault) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.onPrimaryContainer,
					modifier = Modifier.weight(1f),
					textAlign = TextAlign.Center,
				)

				Text(
					text = stringResource(R.string.mixnet_tuning_delays_high),
					style = MaterialTheme.typography.bodyMedium,
					color = MaterialTheme.colorScheme.onPrimaryContainer,
					modifier = Modifier.weight(1f),
					textAlign = TextAlign.End,
				)
			}
		}
	}
}
