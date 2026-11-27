package net.nymtech.nymvpn.ui.screens.main.components

import androidx.compose.animation.core.Animatable
import androidx.compose.animation.core.FastOutSlowInEasing
import androidx.compose.animation.core.InfiniteRepeatableSpec
import androidx.compose.animation.core.LinearEasing
import androidx.compose.animation.core.RepeatMode
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.infiniteRepeatable
import androidx.compose.animation.core.tween
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.size
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.SideEffect
import androidx.compose.runtime.State
import androidx.compose.runtime.remember
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.drawWithCache
import androidx.compose.ui.geometry.Rect
import androidx.compose.ui.geometry.center
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.drawscope.DrawScope
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.ui.model.ConnectionState
import net.nymtech.nymvpn.ui.theme.LocalNymColors
import net.nymtech.vpn.backend.Tunnel

private const val OUTER_RADIUS = 82.4f
private const val MIDDLE_RADIUS = 68.4f
private const val INNER_RADIUS = 54.4f
private const val ARC_STROKE = 5.5f
private val RADII = floatArrayOf(OUTER_RADIUS, MIDDLE_RADIUS, INNER_RADIUS)
private const val OUTER = 0

internal const val FACE_SIZE = (INNER_RADIUS - ARC_STROKE - 6f) * 2f

private const val TOP = -90f
private const val MIN_SWEEP = 0.5f

// error: every ring loses a segment centred on 3 o'clock
private const val ERROR_GAP = 0.1f
private const val ERROR_START = 360f * ERROR_GAP / 2f
private const val ERROR_SWEEP = 360f * (1f - ERROR_GAP)
private const val ERROR_ALPHA = 0.85f

// blocked: the outer ring only, with a gap turning once per BLOCKED_TURN_MS
private const val BLOCKED_SWEEP = 360f * (1f - 0.15f)
private const val BLOCKED_TURN_MS = 4000

private const val STOPPING_ALPHA = 0.15f
private const val STOPPING_FADE_MS = 600

// glow layers: alpha scale and stroke width scale
private val GLOW_ALPHAS = floatArrayOf(0.12f, 0.3f)
private val GLOW_WIDTHS = floatArrayOf(5f, 2.5f)

/**
 * Animated values are read in the draw phase only, so animations redraw without recomposing. Each loop runs only in
 * the state that shows it, so a settled marker requests no frames.
 */
@Composable
internal fun ConnectionRings(connectionState: ConnectionState, isBlocked: Boolean, vpnMode: Tunnel.Mode, animationsOff: Boolean, modifier: Modifier = Modifier) {
	val nymColors = LocalNymColors.current
	val track = nymColors.markerTrack
	val isFailure = connectionState.isFailure
	val isConnected = connectionState == ConnectionState.Connected
	val isStopping = connectionState == ConnectionState.Disconnecting

	// stopping keeps the last fills and fades them out
	val lastFills = remember { RingFillsHolder() }
	val fills = if (isStopping) lastFills.value else ringFills(connectionState, isBlocked)
	SideEffect { if (!isStopping) lastFills.value = fills }

	val sweepSpec = tween<Float>(if (vpnMode.isTwoHop()) 800 else 1200, easing = FastOutSlowInEasing)
	val sweeps = listOf(
		animateFloatAsState(fills.outer, sweepSpec, label = "outerSweep"),
		animateFloatAsState(fills.middle, sweepSpec, label = "middleSweep"),
		animateFloatAsState(fills.inner, sweepSpec, label = "innerSweep"),
	)
	val fillAlpha = animateFloatAsState(
		targetValue = if (isStopping) STOPPING_ALPHA else 1f,
		animationSpec = tween(if (isStopping) STOPPING_FADE_MS else 200, easing = FastOutSlowInEasing),
		label = "fillAlpha",
	)
	val blockedTurn = animateLoopAsState(isBlocked && !animationsOff, 0f, 360f, infiniteRepeatable(tween(BLOCKED_TURN_MS, easing = LinearEasing), RepeatMode.Restart))
	val connectedGlow = animateLoopAsState(isConnected && !animationsOff, 0.10f, 0.32f, infiniteRepeatable(tween(1400, easing = FastOutSlowInEasing), RepeatMode.Reverse))
	val stoppingGlow = animateLoopAsState(isStopping && !animationsOff, 0.04f, 0.22f, infiniteRepeatable(tween(900, easing = FastOutSlowInEasing), RepeatMode.Reverse))

	val baseFill = when {
		isFailure || isBlocked -> nymColors.markerError.copy(alpha = ERROR_ALPHA)
		connectionState is ConnectionState.Connecting -> nymColors.markerConnecting
		else -> MaterialTheme.colorScheme.primary
	}

	Spacer(
		modifier = modifier
			.size((OUTER_RADIUS * 2 + ARC_STROKE).dp)
			.drawWithCache {
				val strokeWidth = ARC_STROKE.dp.toPx()
				val round = Stroke(strokeWidth, cap = StrokeCap.Round)
				val butt = Stroke(strokeWidth, cap = StrokeCap.Butt)
				val glows = Array(GLOW_WIDTHS.size) { Stroke(strokeWidth * GLOW_WIDTHS[it]) }
				val bounds = Array(RADII.size) { Rect(center = size.center, radius = RADII[it].dp.toPx()) }

				onDrawBehind {
					val fillColor = if (isStopping) baseFill.copy(alpha = fillAlpha.value) else baseFill
					val blockedStart = TOP + blockedTurn.value

					if (isStopping) bounds.forEach { ringGlow(it, fillColor, 360f, stoppingGlow.value, glows) }

					bounds.forEachIndexed { i, ring ->
						when {
							isFailure -> arc(ring, track, ERROR_START, ERROR_SWEEP, round)
							isBlocked && i == OUTER -> arc(ring, track, blockedStart, BLOCKED_SWEEP, round)
							else -> arc(ring, track, TOP, 360f, butt)
						}
					}

					if (isConnected) bounds.forEachIndexed { i, ring -> ringGlow(ring, fillColor, sweeps[i].value, connectedGlow.value, glows) }

					bounds.forEachIndexed { i, ring ->
						val sweep = sweeps[i].value
						when {
							isFailure -> arc(ring, fillColor, ERROR_START, ERROR_SWEEP, round)
							sweep <= MIN_SWEEP -> Unit
							isBlocked && i == OUTER -> arc(ring, fillColor, blockedStart, minOf(sweep, BLOCKED_SWEEP), round)
							else -> arc(ring, fillColor, TOP, sweep, round)
						}
					}
				}
			},
	)
}

/** Loops between [from] and [to] while [running]; otherwise rests at [from] and requests no frames. */
@Composable
private fun animateLoopAsState(running: Boolean, from: Float, to: Float, spec: InfiniteRepeatableSpec<Float>): State<Float> {
	val value = remember { Animatable(from) }
	LaunchedEffect(running) {
		value.snapTo(from)
		if (running) value.animateTo(to, spec)
	}
	return value.asState()
}

private fun DrawScope.arc(bounds: Rect, color: Color, startAngle: Float, sweepAngle: Float, stroke: Stroke) {
	drawArc(color = color, startAngle = startAngle, sweepAngle = sweepAngle, useCenter = false, topLeft = bounds.topLeft, size = bounds.size, style = stroke)
}

private fun DrawScope.ringGlow(bounds: Rect, color: Color, sweepAngle: Float, intensity: Float, strokes: Array<Stroke>) {
	if (sweepAngle <= MIN_SWEEP || intensity <= 0.01f) return
	for (i in strokes.indices) {
		arc(bounds, color.copy(alpha = minOf(1f, color.alpha * intensity * GLOW_ALPHAS[i])), TOP, sweepAngle, strokes[i])
	}
}

private data class RingFills(val outer: Float, val middle: Float, val inner: Float) {
	companion object {
		val EMPTY = RingFills(0f, 0f, 0f)
		val FULL = RingFills(360f, 360f, 360f)
	}
}

// plain holder, not state: carries the last live fills into the stopping fade
private class RingFillsHolder(var value: RingFills = RingFills.EMPTY)

private fun ringFills(connectionState: ConnectionState, isBlocked: Boolean): RingFills = when {
	isBlocked -> RingFills(360f, 0f, 0f)
	connectionState is ConnectionState.Connecting -> connectionState.step.let { RingFills(ringSweep(it, 0), ringSweep(it, 1), ringSweep(it, 2)) }
	connectionState == ConnectionState.Connected || connectionState.isFailure -> RingFills.FULL
	else -> RingFills.EMPTY
}

// rings fill outer first, each half then full: step 1 = outer half, 2 = outer full, ..., 6 = inner full
private fun ringSweep(step: Int, ring: Int): Float = when {
	step >= ring * 2 + 2 -> 360f
	step == ring * 2 + 1 -> 180f
	else -> 0f
}
