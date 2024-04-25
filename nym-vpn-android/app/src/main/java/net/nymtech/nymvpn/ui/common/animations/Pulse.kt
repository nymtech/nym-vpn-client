package net.nymtech.nymvpn.ui.common.animations

import androidx.compose.animation.core.InfiniteRepeatableSpec
import androidx.compose.animation.core.RepeatMode.Restart
import androidx.compose.animation.core.StartOffset
import androidx.compose.animation.core.animateFloat
import androidx.compose.animation.core.rememberInfiniteTransition
import androidx.compose.animation.core.tween
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.layout.onGloballyPositioned
import androidx.compose.ui.unit.IntSize
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.ui.theme.CustomColors

@Composable
fun Pulse() {
	MultiplePulsarEffect { modifier ->
		Canvas(modifier = modifier.size(5.dp), onDraw = {
			drawCircle(color = CustomColors.disconnect)
		})
	}
}

@Composable
fun MultiplePulsarEffect(
	nbPulsar: Int = 2,
	pulsarRadius: Float = 10f,
	pulsarColor: Color = CustomColors.disconnect,
	circle: @Composable (Modifier) -> Unit = {},
) {
	var circleSize by remember { mutableStateOf(IntSize(0, 0)) }

	val effects: List<Pair<Float, Float>> = List(nbPulsar) {
		pulsarBuilder(pulsarRadius = pulsarRadius, size = circleSize.width, delay = it * 500)
	}

	Box(
		Modifier,
		contentAlignment = Alignment.Center,
	) {
		Canvas(Modifier, onDraw = {
			for (i in 0 until nbPulsar) {
				val (radius, alpha) = effects[i]
				drawCircle(color = pulsarColor, radius = radius, alpha = alpha)
			}
		})
		circle(
			Modifier
				.padding((pulsarRadius).dp)
				.onGloballyPositioned {
					if (it.isAttached) {
						circleSize = it.size
					}
				},
		)
	}
}

@Composable
fun pulsarBuilder(pulsarRadius: Float, size: Int, delay: Int): Pair<Float, Float> {
	val infiniteTransition = rememberInfiniteTransition(label = "infinite")

	val radius by infiniteTransition.animateFloat(
		initialValue = (size / 2).toFloat(),
		targetValue = size + (pulsarRadius * 2),
		animationSpec = InfiniteRepeatableSpec(
			animation = tween(3000),
			initialStartOffset = StartOffset(delay),
			repeatMode = Restart,
		),
		label = "radius",
	)
	val alpha by infiniteTransition.animateFloat(
		initialValue = 1f,
		targetValue = 0f,
		animationSpec = InfiniteRepeatableSpec(
			animation = tween(3000),
			initialStartOffset = StartOffset(delay + 100),
			repeatMode = Restart,
		),
		label = "alpha",
	)

	return radius to alpha
}
