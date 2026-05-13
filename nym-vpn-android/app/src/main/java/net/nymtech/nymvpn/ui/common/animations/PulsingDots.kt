package net.nymtech.nymvpn.ui.common.animations

import androidx.compose.animation.core.LinearEasing
import androidx.compose.animation.core.animateFloat
import androidx.compose.animation.core.infiniteRepeatable
import androidx.compose.animation.core.rememberInfiniteTransition
import androidx.compose.animation.core.tween
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp

@Composable
fun PulsingDotsWave(modifier: Modifier = Modifier, dotCount: Int = 4, dotSize: Dp = 8.dp, dotColor: Color = MaterialTheme.colorScheme.primary, spaceBetween: Dp = 2.dp, pulseDuration: Int = 1000) {
	val infiniteTransition = rememberInfiniteTransition(label = "waveTransition")
	val progress by infiniteTransition.animateFloat(
		initialValue = 0f,
		targetValue = 1f,
		animationSpec = infiniteRepeatable(
			animation = tween(durationMillis = pulseDuration, easing = LinearEasing),
		),
		label = "waveProgress",
	)

	Row(
		modifier = modifier,
		horizontalArrangement = Arrangement.spacedBy(spaceBetween),
		verticalAlignment = Alignment.CenterVertically,
	) {
		repeat(dotCount) { index ->
			val phase = (progress - index.toFloat() / dotCount + 1f) % 1f
			val scale = if (phase < 0.5f) {
				lerp(0.6f, 1.2f, phase * 2)
			} else {
				lerp(1.2f, 0.6f, (phase - 0.5f) * 2)
			}

			Box(
				modifier = Modifier
					.size(dotSize)
					.graphicsLayer {
						scaleX = scale
						scaleY = scale
					}
					.background(dotColor, CircleShape),
			)
		}
	}
}

private fun lerp(start: Float, stop: Float, fraction: Float): Float = start + (stop - start) * fraction

@Preview(showBackground = true)
@Composable
private fun PulsingDotsWavePreview() {
	Box(
		modifier = Modifier
			.fillMaxSize()
			.background(Color(0xFF123D2C)),
		contentAlignment = Alignment.Center,
	) {
		PulsingDotsWave()
	}
}
