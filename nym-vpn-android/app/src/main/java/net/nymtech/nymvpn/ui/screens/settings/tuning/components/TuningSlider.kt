package net.nymtech.nymvpn.ui.screens.settings.tuning.components

import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Slider
import androidx.compose.material3.SliderDefaults
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.DpSize
import androidx.compose.ui.unit.dp

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun TuningSlider(value: Float, onValueChange: (Float) -> Unit, valueRange: ClosedFloatingPointRange<Float>, interactionSource: MutableInteractionSource, modifier: Modifier = Modifier) {
	Slider(
		value = value,
		onValueChange = onValueChange,
		valueRange = valueRange,
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
