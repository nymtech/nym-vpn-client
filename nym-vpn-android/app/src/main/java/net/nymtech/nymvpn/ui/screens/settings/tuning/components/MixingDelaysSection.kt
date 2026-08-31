package net.nymtech.nymvpn.ui.screens.settings.tuning.components

import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.theme.LocalNymColors
import kotlin.math.roundToInt

@Composable
fun MixingDelaysSection(delayValue: Float, valueRange: ClosedFloatingPointRange<Float>, defaultValue: Float, onDelayValueChange: (Float) -> Unit) {
	val interactionSource = remember { MutableInteractionSource() }

	val currentInt = delayValue.roundToInt()
	val isDefault = currentInt == defaultValue.roundToInt()

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
					text = stringResource(R.string.mixnet_tuning_delays_warning),
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

			SliderRangeLabels()

			TuningSlider(
				value = delayValue,
				onValueChange = { newValue ->
					onDelayValueChange(newValue.roundToInt().toFloat())
				},
				valueRange = valueRange,
				interactionSource = interactionSource,
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
					color = if (isDefault) MaterialTheme.colorScheme.primary else LocalNymColors.current.currentValueIndicator,
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
