package net.nymtech.nymvpn.ui.screens.main.components

import android.content.Context
import android.provider.Settings
import androidx.compose.animation.core.FastOutSlowInEasing
import androidx.compose.animation.core.RepeatMode
import androidx.compose.animation.core.animateFloat
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.infiniteRepeatable
import androidx.compose.animation.core.rememberInfiniteTransition
import androidx.compose.animation.core.tween
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.alpha
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.geometry.Size
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.graphics.drawscope.DrawScope
import androidx.compose.ui.graphics.drawscope.Stroke
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.common.animations.Pulse
import net.nymtech.nymvpn.ui.model.ConnectionState
import net.nymtech.nymvpn.ui.model.Marker
import net.nymtech.nymvpn.ui.model.MarkerMode
import net.nymtech.nymvpn.ui.model.MarkerState
import net.nymtech.nymvpn.ui.theme.LocalNymColors
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.util.StringValue
import net.nymtech.vpn.backend.Tunnel
import nym_vpn_lib_types.ErrorStateReason

private const val OUTER_RADIUS = 82.4f
private const val MIDDLE_RADIUS = 68.4f
private const val INNER_RADIUS = 54.4f
private const val ARC_STROKE = 5.5f
private const val LABEL_OFFSET = 14f

// the face sits inside the inner ring with a small gap
private const val FACE_SIZE = (INNER_RADIUS - ARC_STROKE - 6f) * 2f

// Error rings each lose a segment centred on 3 o'clock (0 degrees here), so the state reads without colour.
private const val ERROR_GAP = 0.1f
private const val ERROR_START = 360f * ERROR_GAP / 2f
private const val ERROR_SWEEP = 360f * (1f - ERROR_GAP)
private const val ERROR_ALPHA = 0.85f

// Blocked: only the outer ring, blinking.
private const val BLOCKED_ALPHA_MIN = 0.15f
private const val BLOCKED_ALPHA_MAX = 0.9f
private const val BLOCKED_HALF_CYCLE_MS = 1200

@Composable
fun ConnectionStatus(connectionState: ConnectionState, marker: Marker, vpnMode: Tunnel.Mode, connectionTime: String? = null, modifier: Modifier = Modifier) {
	val context = LocalContext.current
	val bg = MaterialTheme.colorScheme.surface
	val animationsOff = remember(context) { areAnimationsOff(context) }

	// a failure reported by the backend reads as an error even once the tunnel is down
	val phase = if (connectionState is ConnectionState.Error || connectionState is ConnectionState.StartFailure) MarkerState.ERROR else marker.state
	val isError = phase == MarkerState.ERROR
	val isConnected = phase == MarkerState.CONNECTED
	val isCanceling = phase == MarkerState.STOPPING
	val isBlocked = phase == MarkerState.BLOCKED
	val isConnecting = phase == MarkerState.CONNECTING

	val (targetOuter, targetMiddle, targetInner) = remember(phase, marker.step) {
		ringFillTargets(phase, marker.step)
	}
	val sweepMs = if (vpnMode.isTwoHop()) 800 else 1200

	val outerSweep by animateFloatAsState(
		targetValue = targetOuter,
		animationSpec = tween(sweepMs, easing = FastOutSlowInEasing),
		label = "outerSweep",
	)
	val middleSweep by animateFloatAsState(
		targetValue = targetMiddle,
		animationSpec = tween(sweepMs, easing = FastOutSlowInEasing),
		label = "middleSweep",
	)
	val innerSweep by animateFloatAsState(
		targetValue = targetInner,
		animationSpec = tween(sweepMs, easing = FastOutSlowInEasing),
		label = "innerSweep",
	)

	val pulse = rememberInfiniteTransition(label = "arcPulse")

	// Blocked
	val blockedBlink by pulse.animateFloat(
		initialValue = BLOCKED_ALPHA_MIN,
		targetValue = BLOCKED_ALPHA_MAX,
		animationSpec = infiniteRepeatable(
			animation = tween(BLOCKED_HALF_CYCLE_MS, easing = FastOutSlowInEasing),
			repeatMode = RepeatMode.Reverse,
		),
		label = "blockedBlink",
	)

	// Connected
	val connectedGlow by pulse.animateFloat(
		initialValue = 0.10f,
		targetValue = 0.32f,
		animationSpec = infiniteRepeatable(
			animation = tween(1400, easing = FastOutSlowInEasing),
			repeatMode = RepeatMode.Reverse,
		),
		label = "connectedGlow",
	)

	// Canceling
	val cancelingGlow by pulse.animateFloat(
		initialValue = 0.04f,
		targetValue = 0.22f,
		animationSpec = infiniteRepeatable(
			animation = tween(900, easing = FastOutSlowInEasing),
			repeatMode = RepeatMode.Reverse,
		),
		label = "cancelingGlow",
	)

	val fillColor = when {
		isError -> MaterialTheme.colorScheme.error.copy(alpha = ERROR_ALPHA)
		isConnecting -> LocalNymColors.current.markerConnecting
		isBlocked -> MaterialTheme.colorScheme.primary.copy(alpha = if (animationsOff) BLOCKED_ALPHA_MAX else blockedBlink)
		else -> MaterialTheme.colorScheme.primary
	}

	val connectedLabel = if (vpnMode.isTwoHop()) stringResource(R.string.connection_status_fast_mode) else stringResource(R.string.connection_status_anonymous)
	val failedLabel = stringResource(R.string.connection_failed)
	val disconnectingLabel = stringResource(R.string.disconnecting)
	val notProtectedLabel = stringResource(R.string.connection_status_not_protected)
	val offlineLabel = stringResource(R.string.offline)
	val blockedLabel = stringResource(R.string.connection_status_blocked)
	val reconnectLine = stringResource(R.string.connection_status_offline_reconnect)

	val isOffline = connectionState == ConnectionState.Offline || connectionState == ConnectionState.WaitingForConnection
	val currentLabel: String? = when {
		isBlocked -> blockedLabel
		phase == MarkerState.NOT_PROTECTED && isOffline -> notProtectedLabel
		else -> when (connectionState) {
			ConnectionState.Connected -> connectedLabel
			is ConnectionState.Connecting -> connectionState.label.asString(context)
			is ConnectionState.Error, is ConnectionState.StartFailure -> failedLabel
			ConnectionState.Disconnecting -> disconnectingLabel
			ConnectionState.Offline, ConnectionState.WaitingForConnection -> offlineLabel
			ConnectionState.Disconnected -> notProtectedLabel
		}
	}
	// blocked: will connect once online; offline with no session: not protected, no internet
	val offlineLine: String? = when {
		isBlocked -> reconnectLine
		phase == MarkerState.NOT_PROTECTED && connectionState == ConnectionState.WaitingForConnection -> reconnectLine
		phase == MarkerState.NOT_PROTECTED && connectionState == ConnectionState.Offline -> offlineLabel
		else -> null
	}
	val showPulse = isOffline && offlineLine == null

	val labelAlpha by animateFloatAsState(
		targetValue = if (currentLabel != null) 1f else 0f,
		animationSpec = tween(250, easing = FastOutSlowInEasing),
		label = "labelAlpha",
	)
	val timeAlpha by animateFloatAsState(
		targetValue = if (connectionTime != null || offlineLine != null) 1f else 0f,
		animationSpec = tween(250, easing = FastOutSlowInEasing),
		label = "timeAlpha",
	)

	Column(
		horizontalAlignment = Alignment.CenterHorizontally,
		modifier = modifier,
	) {
		Box(contentAlignment = Alignment.Center) {
			Canvas(modifier = Modifier.size((OUTER_RADIUS * 2 + ARC_STROKE).dp)) {
				val cx = size.width / 2f
				val cy = size.height / 2f

				if (isCanceling) {
					for (radius in floatArrayOf(OUTER_RADIUS, MIDDLE_RADIUS, INNER_RADIUS)) {
						val rPx = radius.dp.toPx()
						drawArcGlow(
							color = fillColor,
							sweepAngle = 360f,
							topLeft = Offset(cx - rPx, cy - rPx),
							size = Size(rPx * 2f, rPx * 2f),
							strokePx = ARC_STROKE.dp.toPx(),
							glowIntensity = cancelingGlow,
						)
					}
				}

				for (radius in floatArrayOf(OUTER_RADIUS, MIDDLE_RADIUS, INNER_RADIUS)) {
					val rPx = radius.dp.toPx()
					drawArc(
						color = bg,
						startAngle = if (isError) ERROR_START else -90f,
						sweepAngle = if (isError) ERROR_SWEEP else 360f,
						useCenter = false,
						topLeft = Offset(cx - rPx, cy - rPx),
						size = Size(rPx * 2f, rPx * 2f),
						style = Stroke(width = ARC_STROKE.dp.toPx(), cap = if (isError) StrokeCap.Round else StrokeCap.Butt),
					)
				}

				if (isConnected) {
					for ((radius, sweep) in listOf(
						OUTER_RADIUS to outerSweep,
						MIDDLE_RADIUS to middleSweep,
						INNER_RADIUS to innerSweep,
					)) {
						if (sweep > 0.5f) {
							val rPx = radius.dp.toPx()
							drawArcGlow(
								color = fillColor,
								sweepAngle = sweep,
								topLeft = Offset(cx - rPx, cy - rPx),
								size = Size(rPx * 2f, rPx * 2f),
								strokePx = ARC_STROKE.dp.toPx(),
								glowIntensity = connectedGlow,
							)
						}
					}
				}

				for ((radius, sweep) in listOf(
					OUTER_RADIUS to outerSweep,
					MIDDLE_RADIUS to middleSweep,
					INNER_RADIUS to innerSweep,
				)) {
					val rPx = radius.dp.toPx()
					if (isError) {
						drawArc(
							color = fillColor,
							startAngle = ERROR_START,
							sweepAngle = ERROR_SWEEP,
							useCenter = false,
							topLeft = Offset(cx - rPx, cy - rPx),
							size = Size(rPx * 2f, rPx * 2f),
							style = Stroke(width = ARC_STROKE.dp.toPx(), cap = StrokeCap.Round),
						)
					} else if (sweep > 0.5f) {
						drawArc(
							color = fillColor,
							startAngle = -90f,
							sweepAngle = sweep,
							useCenter = false,
							topLeft = Offset(cx - rPx, cy - rPx),
							size = Size(rPx * 2f, rPx * 2f),
							style = Stroke(width = ARC_STROKE.dp.toPx(), cap = StrokeCap.Round),
						)
					}
				}
			}

			// Face: says the mode (two blocks for dVPN, the mix field for Mixnet), centred in the inner disk
			MarkerFace(
				mode = marker.mode,
				lit = phase != MarkerState.NOT_PROTECTED,
				playing = isConnected && !animationsOff,
				litColor = MaterialTheme.colorScheme.onPrimaryContainer,
				unlitColor = bg,
				modifier = Modifier.size(FACE_SIZE.dp),
			)
		}

		// State label — always takes space to keep the circle position fixed
		Column(
			horizontalAlignment = Alignment.CenterHorizontally,
			modifier = Modifier.alpha(labelAlpha),
		) {
			Spacer(Modifier.height(LABEL_OFFSET.dp))
			Row(
				verticalAlignment = Alignment.CenterVertically,
				horizontalArrangement = Arrangement.Center,
			) {
				if (showPulse) {
					Pulse(color = MaterialTheme.colorScheme.error)
					Spacer(Modifier.width(6.dp))
				}
				Text(
					text = (currentLabel ?: "").uppercase(),
					style = MaterialTheme.typography.labelSmall,
					color = when {
						isError -> MaterialTheme.colorScheme.error
						isConnected -> MaterialTheme.colorScheme.primary
						showPulse -> MaterialTheme.colorScheme.error
						else -> MaterialTheme.colorScheme.onSurfaceVariant
					},
				)
			}
			Spacer(Modifier.height(8.dp))
			Text(
				text = offlineLine ?: connectionTime ?: "",
				style = MaterialTheme.typography.labelSmall,
				color = if (offlineLine != null) MaterialTheme.colorScheme.onSurfaceVariant else MaterialTheme.colorScheme.primary,
				modifier = Modifier.alpha(timeAlpha),
			)
		}
	}
}

/** Animator duration scale 0 (developer options or battery saver): hold animations still. */
private fun areAnimationsOff(context: Context): Boolean = Settings.Global.getFloat(context.contentResolver, Settings.Global.ANIMATOR_DURATION_SCALE, 1f) == 0f

// Layered stroke glow
private fun DrawScope.drawArcGlow(color: Color, sweepAngle: Float, topLeft: Offset, size: Size, strokePx: Float, glowIntensity: Float) {
	if (sweepAngle <= 0.5f || glowIntensity <= 0.01f) return
	val base = color.alpha
	drawArc(
		color = color.copy(alpha = minOf(1f, base * glowIntensity * 0.12f)),
		startAngle = -90f,
		sweepAngle = sweepAngle,
		useCenter = false,
		topLeft = topLeft,
		size = size,
		style = Stroke(width = strokePx * 5f),
	)
	drawArc(
		color = color.copy(alpha = minOf(1f, base * glowIntensity * 0.3f)),
		startAngle = -90f,
		sweepAngle = sweepAngle,
		useCenter = false,
		topLeft = topLeft,
		size = size,
		style = Stroke(width = strokePx * 2.5f),
	)
}

private data class RingFillTargets(val outer: Float, val middle: Float, val inner: Float)

// Connecting fills ring by ring, outer first, each half then sealed: step 1 = outer half, 2 = outer full, ... 6 = inner full.
private fun ringFillTargets(phase: MarkerState, step: Int?): RingFillTargets = when (phase) {
	MarkerState.CONNECTED, MarkerState.ERROR -> RingFillTargets(360f, 360f, 360f)
	MarkerState.BLOCKED -> RingFillTargets(360f, 0f, 0f)
	MarkerState.CONNECTING -> {
		val current = (step ?: 1).coerceIn(1, Marker.CONNECTING_STEP_COUNT)
		RingFillTargets(ringSweep(current, 0), ringSweep(current, 1), ringSweep(current, 2))
	}
	MarkerState.NOT_PROTECTED, MarkerState.STOPPING -> RingFillTargets(0f, 0f, 0f)
}

private fun ringSweep(step: Int, ring: Int): Float = when {
	step >= ring * 2 + 2 -> 360f
	step == ring * 2 + 1 -> 180f
	else -> 0f
}

@Composable
@Preview(showBackground = true)
private fun ArcPreviewConnectedDark() {
	NymVPNTheme(Theme.DARK_MODE) {
		Surface(modifier = Modifier.background(MaterialTheme.colorScheme.background)) {
			ConnectionStatus(
				connectionState = ConnectionState.Disconnected,
				marker = Marker(MarkerState.NOT_PROTECTED, MarkerMode.DVPN),
				vpnMode = Tunnel.Mode.TWO_HOP_MIXNET,
				connectionTime = "2.2.2.2",
			)
		}
	}
}

@Composable
@Preview(showBackground = true)
private fun ArcPreviewConnected() {
	NymVPNTheme(Theme.LIGHT_MODE) {
		Surface(modifier = Modifier.background(MaterialTheme.colorScheme.background)) {
			ConnectionStatus(
				connectionState = ConnectionState.Connected,
				marker = Marker(MarkerState.CONNECTED, MarkerMode.DVPN),
				vpnMode = Tunnel.Mode.TWO_HOP_MIXNET,
				connectionTime = "2.2.2.2",
			)
		}
	}
}

@Composable
@Preview(showBackground = true)
private fun ArcPreviewError() {
	NymVPNTheme(Theme.DARK_MODE) {
		ConnectionStatus(
			connectionState = ConnectionState.Error(ErrorStateReason.InactiveSubscription),
			marker = Marker(MarkerState.ERROR, MarkerMode.DVPN),
			vpnMode = Tunnel.Mode.TWO_HOP_MIXNET,
		)
	}
}

@Composable
@Preview(showBackground = true)
private fun ArcPreviewConnecting() {
	NymVPNTheme(Theme.DARK_MODE) {
		ConnectionStatus(
			connectionState = ConnectionState.Connecting(StringValue.DynamicString("Selecting gateways")),
			marker = Marker(MarkerState.CONNECTING, MarkerMode.MIXNET, step = 4),
			vpnMode = Tunnel.Mode.FIVE_HOP_MIXNET,
		)
	}
}

@Composable
@Preview(showBackground = true)
private fun ArcPreviewBlocked() {
	NymVPNTheme(Theme.LIGHT_MODE) {
		ConnectionStatus(
			connectionState = ConnectionState.WaitingForConnection,
			marker = Marker(MarkerState.BLOCKED, MarkerMode.MIXNET),
			vpnMode = Tunnel.Mode.FIVE_HOP_MIXNET,
		)
	}
}
