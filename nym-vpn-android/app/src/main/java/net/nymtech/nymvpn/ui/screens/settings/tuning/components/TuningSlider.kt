package net.nymtech.nymvpn.ui.screens.settings.tuning.components

import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Slider
import androidx.compose.material3.SliderDefaults
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.DpSize
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import kotlin.math.roundToInt

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun TuningSlider(
	value: Float,
	onValueChange: (Float) -> Unit,
	valueRange: ClosedFloatingPointRange<Float>,
	interactionSource: MutableInteractionSource,
	modifier: Modifier = Modifier,
	steps: Int = 0,
) {
	Slider(
		value = value,
		onValueChange = onValueChange,
		valueRange = valueRange,
		steps = steps,
		interactionSource = interactionSource,
		modifier = modifier.fillMaxWidth(),
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

/**
 * A [TuningSlider] that snaps to one of [items], reporting the selected item rather than a raw Float.
 */
@Composable
fun <T> DiscreteTuningSlider(items: List<T>, selected: T, onSelectedChange: (T) -> Unit, interactionSource: MutableInteractionSource, modifier: Modifier = Modifier) {
	val selectedIndex = items.indexOf(selected).coerceAtLeast(0)
	TuningSlider(
		value = selectedIndex.toFloat(),
		onValueChange = { onSelectedChange(items[it.roundToInt().coerceIn(items.indices)]) },
		valueRange = 0f..(items.size - 1).toFloat(),
		steps = items.size - 2,
		interactionSource = interactionSource,
		modifier = modifier,
	)
}

/** The "Faster" / "+ Anonymity" labels shown above every tuning slider on this screen. */
@Composable
fun SliderRangeLabels(modifier: Modifier = Modifier) {
	Row(
		modifier = modifier.fillMaxWidth(),
		horizontalArrangement = Arrangement.SpaceBetween,
	) {
		Text(
			text = stringResource(R.string.mixnet_tuning_slider_faster_label),
			style = MaterialTheme.typography.bodySmall,
			color = MaterialTheme.colorScheme.onBackground,
		)
		Text(
			text = stringResource(R.string.mixnet_tuning_slider_anonymity_label),
			style = MaterialTheme.typography.bodySmall,
			color = MaterialTheme.colorScheme.onBackground,
		)
	}
}

/** Bottom labels for a [DiscreteTuningSlider], highlighting whichever index is currently selected. */
@Composable
fun DiscreteRateLabels(labels: List<String>, selectedIndex: Int, modifier: Modifier = Modifier) {
	Row(
		modifier = modifier.fillMaxWidth(),
		horizontalArrangement = Arrangement.SpaceBetween,
	) {
		labels.forEachIndexed { index, label ->
			Text(
				text = label,
				style = MaterialTheme.typography.bodyMedium,
				color = if (index == selectedIndex) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.onPrimaryContainer,
				modifier = Modifier.weight(1f),
				textAlign = when (index) {
					0 -> TextAlign.Start
					labels.lastIndex -> TextAlign.End
					else -> TextAlign.Center
				},
			)
		}
	}
}
