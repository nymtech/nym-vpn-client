package net.nymtech.nymvpn.ui.screens.settings.tuning.components

import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
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
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.DpSize
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.common.buttons.ScaledSwitch
import net.nymtech.nymvpn.ui.theme.LocalNymColors
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun SendTrafficSection(trafficEnabled: Boolean, onTrafficEnable: (enabled: Boolean) -> Unit, trafficValue: Float, onTrafficValueChange: (Float) -> Unit) {
	val interactionSource = remember { MutableInteractionSource() }

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
			Row(
				modifier = Modifier.fillMaxWidth(),
				horizontalArrangement = Arrangement.SpaceBetween,
				verticalAlignment = Alignment.CenterVertically,
			) {
				Text(
					text = stringResource(R.string.mixnet_tuning_traffic_title),
					style = MaterialTheme.typography.titleMedium,
					color = MaterialTheme.colorScheme.onPrimaryContainer,
					maxLines = 2,
					overflow = TextOverflow.Ellipsis,
					modifier = Modifier.weight(1f),
				)
				Spacer(modifier = Modifier.width(8.dp))
				ScaledSwitch(
					checked = trafficEnabled,
					onClick = { onTrafficEnable(it) },
				)
			}

			if (!trafficEnabled) {
				Text(
					text = stringResource(R.string.mixnet_tuning_traffic_warning),
					style = MaterialTheme.typography.bodySmall,
					color = LocalNymColors.current.warning,
				)
				Text(
					text = stringResource(R.string.mixnet_tuning_traffic_off_title),
					color = MaterialTheme.colorScheme.onPrimaryContainer,
					style = MaterialTheme.typography.titleMedium,
					maxLines = 2,
					overflow = TextOverflow.Ellipsis,
				)
			}

			Text(
				text = stringResource(if (trafficEnabled) R.string.mixnet_tuning_traffic_on_description else R.string.mixnet_tuning_traffic_off_description),
				style = MaterialTheme.typography.bodySmall,
				color = MaterialTheme.colorScheme.onBackground,
			)

			Column(
				modifier = Modifier.fillMaxWidth(),
			) {
				Row(
					modifier = Modifier.fillMaxWidth(),
					horizontalArrangement = Arrangement.SpaceBetween,
				) {
					Text(
						text = stringResource(R.string.mixnet_tuning_traffic_min),
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
					value = trafficValue,
					onValueChange = onTrafficValueChange,
					valueRange = 0f..200f,
					interactionSource = interactionSource,
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
			}

			Row(
				modifier = Modifier.fillMaxWidth(),
				horizontalArrangement = Arrangement.SpaceBetween,
			) {
				Text(
					text = stringResource(if (trafficEnabled) R.string.mixnet_tuning_traffic_on_low else R.string.mixnet_tuning_traffic_off_low),
					style = MaterialTheme.typography.bodyMedium,
					color = MaterialTheme.colorScheme.onPrimaryContainer,
					modifier = Modifier.weight(1f),
					textAlign = TextAlign.Start,
				)

				Text(
					text = stringResource(if (trafficEnabled) R.string.mixnet_tuning_traffic_on_balanced else R.string.mixnet_tuning_traffic_off_balanced),
					style = MaterialTheme.typography.bodyMedium,
					color = MaterialTheme.colorScheme.onPrimaryContainer,
					modifier = Modifier.weight(1f),
					textAlign = TextAlign.Center,
				)

				if (!trafficEnabled) {
					Text(
						text = stringResource(R.string.mixnet_tuning_traffic_off_medium),
						style = MaterialTheme.typography.bodyMedium,
						color = MaterialTheme.colorScheme.onPrimaryContainer,
						modifier = Modifier.weight(1f),
						textAlign = TextAlign.Center,
					)
				}
				Text(
					text = stringResource(if (trafficEnabled) R.string.mixnet_tuning_traffic_on_high else R.string.mixnet_tuning_traffic_off_high),
					style = MaterialTheme.typography.bodyMedium,
					color = MaterialTheme.colorScheme.onPrimaryContainer,
					modifier = Modifier.weight(1f),
					textAlign = TextAlign.End,
				)
			}
		}
	}
}

@Composable
@Preview
internal fun PreviewMSendTrafficSection() {
	NymVPNTheme(Theme.default()) {
		SendTrafficSection(
			trafficEnabled = false,
			onTrafficEnable = {},
			trafficValue = 10f,
			onTrafficValueChange = {},
		)
	}
}
