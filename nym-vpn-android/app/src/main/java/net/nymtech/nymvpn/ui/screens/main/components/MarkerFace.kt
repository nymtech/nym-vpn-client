package net.nymtech.nymvpn.ui.screens.main.components

import androidx.compose.foundation.Canvas
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.Path
import androidx.compose.ui.graphics.drawscope.DrawScope
import androidx.compose.ui.graphics.drawscope.Stroke
import kotlinx.coroutines.delay
import net.nymtech.nymvpn.ui.model.MarkerMode

// The marker's face says the mode. dVPN: an entry and an exit block, linked (2 hops). Mixnet: the three mix layers as
// a 3x3 field, with the five hops lit (entry, one node per layer, exit); hollow nodes are the unused ones. While not
// protected nothing is lit: dVPN shows only its outline, Mixnet only its nodes. While connected the face plays an
// illustrative route: a packet steps through the hops, and the Mixnet route changes every two passes. It illustrates
// how the network works; it is not a traffic meter. Drawn in an 80 x 80 box.

private const val BOX = 80f
private const val LINE_WIDTH = 2f
private const val NODE_R = 3.5f
private const val NODE_STROKE = 1.5f
private const val BLOCK = 18f
private const val HOP_SQUARE = 10f

private val DVPN_ENTRY = Offset(13f, 31f)
private val DVPN_EXIT = Offset(49f, 31f)
private val DVPN_LINK_START = Offset(31f, 40f)
private val DVPN_LINK_END = Offset(49f, 40f)

private val MIX_ENTRY = Offset(8f, 40f)
private val MIX_EXIT = Offset(72f, 40f)
private val LAYER_X = floatArrayOf(24f, 40f, 56f)
private val LAYER_Y = floatArrayOf(22f, 40f, 58f)
private val MIX_ROUTES: List<List<Offset>> = listOf(
	listOf(MIX_ENTRY, Offset(24f, 22f), Offset(40f, 58f), Offset(56f, 22f), MIX_EXIT),
	listOf(MIX_ENTRY, Offset(24f, 40f), Offset(40f, 22f), Offset(56f, 40f), MIX_EXIT),
	listOf(MIX_ENTRY, Offset(24f, 58f), Offset(40f, 22f), Offset(56f, 58f), MIX_EXIT),
	listOf(MIX_ENTRY, Offset(24f, 40f), Offset(40f, 58f), Offset(56f, 40f), MIX_EXIT),
)

// rhythm: the data line ticks (dVPN 80 ms, Mixnet 120 ms)
private const val DVPN_TICK_MS = 80L
private const val MIX_TICK_MS = 120L
private const val DVPN_TICKS_PER_HOP = 3 // entry, link, exit: 240 ms each
private const val MIX_TICKS_PER_HOP = 2 // 240 ms per hop
private const val MIX_PASS = 6 // five hops and one rest beat
private const val MIX_PASSES_PER_ROUTE = 2
private const val DIM = 0.32f
private const val TRAIL = 0.6f
private const val ROUTE_ALPHA = 0.4f

/**
 * Decorative: no content description. [lit] is false while not protected (drawn in [unlitColor], the ring track grey).
 * [playing] runs the illustrative route; pass false when animations are off so the face holds still, all lit.
 */
@Composable
fun MarkerFace(mode: MarkerMode, lit: Boolean, playing: Boolean, litColor: Color, unlitColor: Color, modifier: Modifier = Modifier) {
	val play = lit && playing
	val tick = rememberTick(play, if (mode == MarkerMode.MIXNET) MIX_TICK_MS else DVPN_TICK_MS)
	val color = if (lit) litColor else unlitColor
	Canvas(modifier = modifier) {
		val scale = size.minDimension / BOX
		when (mode) {
			MarkerMode.DVPN -> drawDvpnFace(color, lit, play, tick, scale)
			MarkerMode.MIXNET -> drawMixnetFace(color, lit, play, tick, scale)
		}
	}
}

// counts ticks while running; resets to 0 when stopped
@Composable
private fun rememberTick(running: Boolean, tickMs: Long): Int {
	var tick by remember { mutableIntStateOf(0) }
	LaunchedEffect(running, tickMs) {
		tick = 0
		while (running) {
			delay(tickMs)
			tick++
		}
	}
	return tick
}

private fun DrawScope.drawDvpnFace(color: Color, lit: Boolean, playing: Boolean, tick: Int, scale: Float) {
	// 0 entry, 1 link, 2 exit; -1 when still
	val hop = if (playing) (tick / DVPN_TICKS_PER_HOP) % 3 else -1
	val alphaOf = { i: Int -> if (hop < 0 || hop == i) 1f else DIM }
	drawLine(color = color, start = DVPN_LINK_START * scale, end = DVPN_LINK_END * scale, strokeWidth = LINE_WIDTH * scale, alpha = alphaOf(1))
	drawSquare(color, DVPN_ENTRY, BLOCK, lit, alphaOf(0), scale)
	drawSquare(color, DVPN_EXIT, BLOCK, lit, alphaOf(2), scale)
}

private fun DrawScope.drawMixnetFace(color: Color, lit: Boolean, playing: Boolean, tick: Int, scale: Float) {
	val step = tick / MIX_TICKS_PER_HOP
	val route = if (playing) MIX_ROUTES[(step / (MIX_PASS * MIX_PASSES_PER_ROUTE)) % MIX_ROUTES.size] else MIX_ROUTES[0]
	// index of the lit hop; -1 when still
	val hop = if (playing) step % MIX_PASS else -1

	if (lit) {
		val path = Path()
		route.forEachIndexed { i, point ->
			if (i == 0) path.moveTo(point.x * scale, point.y * scale) else path.lineTo(point.x * scale, point.y * scale)
		}
		drawPath(path = path, color = color, alpha = if (playing) ROUTE_ALPHA else 1f, style = Stroke(width = LINE_WIDTH * scale))
	} else {
		drawNode(color, MIX_ENTRY, scale)
		drawNode(color, MIX_EXIT, scale)
	}
	for (x in LAYER_X) {
		for (y in LAYER_Y) {
			val onRoute = lit && route.any { it.x == x && it.y == y }
			if (!onRoute) drawNode(color, Offset(x, y), scale)
		}
	}
	if (lit) {
		route.forEachIndexed { i, point ->
			val alpha = when {
				hop < 0 || i == hop -> 1f
				i < hop -> TRAIL
				else -> DIM
			}
			drawSquare(color, Offset(point.x - HOP_SQUARE / 2f, point.y - HOP_SQUARE / 2f), HOP_SQUARE, true, alpha, scale)
		}
	}
}

private fun DrawScope.drawNode(color: Color, center: Offset, scale: Float) {
	drawCircle(color = color, radius = NODE_R * scale, center = center * scale, style = Stroke(width = NODE_STROKE * scale))
}

// A square at [topLeft] with a centred outline; filled, the outline is folded in so one alpha covers both.
private fun DrawScope.drawSquare(color: Color, topLeft: Offset, side: Float, filled: Boolean, alpha: Float, scale: Float) {
	if (filled) {
		val half = LINE_WIDTH / 2f
		drawRect(color = color, topLeft = Offset(topLeft.x - half, topLeft.y - half) * scale, size = Size(side + LINE_WIDTH, side + LINE_WIDTH) * scale, alpha = alpha)
	} else {
		drawRect(color = color, topLeft = topLeft * scale, size = Size(side, side) * scale, alpha = alpha, style = Stroke(width = LINE_WIDTH * scale))
	}
}
