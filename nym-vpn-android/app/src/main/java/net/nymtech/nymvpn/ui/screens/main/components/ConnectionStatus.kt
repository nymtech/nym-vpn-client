package net.nymtech.nymvpn.ui.screens.main.components

import androidx.compose.animation.core.FastOutSlowInEasing
import androidx.compose.animation.core.RepeatMode
import androidx.compose.animation.core.animateFloat
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.infiniteRepeatable
import androidx.compose.animation.core.keyframes
import androidx.compose.animation.core.rememberInfiniteTransition
import androidx.compose.animation.core.tween
import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
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
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.vpn.backend.Tunnel
import nym_vpn_lib_types.EstablishConnectionState

private const val OUTER_RADIUS = 82.4f
private const val MIDDLE_RADIUS = 68.4f
private const val INNER_RADIUS = 54.4f
private const val ARC_STROKE = 5.5f
private const val LABEL_OFFSET = 14f

@Composable
fun ConnectionStatus(
	connectionState: ConnectionState,
	vpnMode: Tunnel.Mode,
	establishConnectionState: EstablishConnectionState? = null,
	connectionTime: String? = null,
	modifier: Modifier = Modifier,
) {
	val context = LocalContext.current
	val bg = MaterialTheme.colorScheme.surface

	val isError = connectionState is ConnectionState.Error || connectionState is ConnectionState.StartFailure
	val isConnected = connectionState == ConnectionState.Connected
	val isCanceling = connectionState == ConnectionState.Disconnecting

	val (targetOuter, targetMiddle, targetInner) = remember(connectionState, establishConnectionState) {
		ringFillTargets(connectionState, establishConnectionState)
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

	// Error
	val errorPulse by pulse.animateFloat(
		initialValue = 0.60f,
		targetValue = 0.90f,
		animationSpec = infiniteRepeatable(
			animation = keyframes {
				durationMillis = 880
				0.60f at 0
				0.90f at 200
				0.90f at 280
				0.60f at 880
			},
			repeatMode = RepeatMode.Restart,
		),
		label = "errorPulse",
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
		isError -> MaterialTheme.colorScheme.error.copy(alpha = errorPulse)
		else -> MaterialTheme.colorScheme.primary
	}

	val connectedLabel = if (vpnMode.isTwoHop()) stringResource(R.string.connection_status_fast_mode) else stringResource(R.string.connection_status_anonymous)
	val failedLabel = stringResource(R.string.connection_failed)
	val disconnectingLabel = stringResource(R.string.disconnecting)
	val notProtectedLabel = stringResource(R.string.connection_status_not_protected)
	val offlineLabel = stringResource(R.string.offline)

	val currentLabel: String? = when (connectionState) {
		ConnectionState.Connected -> connectedLabel
		is ConnectionState.Connecting -> connectionState.label.asString(context)
		is ConnectionState.Error, is ConnectionState.StartFailure -> failedLabel
		ConnectionState.Disconnecting -> disconnectingLabel
		ConnectionState.Offline, ConnectionState.WaitingForConnection -> offlineLabel
		ConnectionState.Disconnected -> notProtectedLabel
	}

	val labelAlpha by animateFloatAsState(
		targetValue = if (currentLabel != null) 1f else 0f,
		animationSpec = tween(250, easing = FastOutSlowInEasing),
		label = "labelAlpha",
	)
	val timeAlpha by animateFloatAsState(
		targetValue = if (connectionTime != null) 1f else 0f,
		animationSpec = tween(250, easing = FastOutSlowInEasing),
		label = "timeAlpha",
	)

	Column(
		horizontalAlignment = Alignment.CenterHorizontally,
		modifier = modifier,
	) {
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
					startAngle = -90f,
					sweepAngle = 360f,
					useCenter = false,
					topLeft = Offset(cx - rPx, cy - rPx),
					size = Size(rPx * 2f, rPx * 2f),
					style = Stroke(width = ARC_STROKE.dp.toPx()),
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
				if (sweep > 0.5f) {
					val rPx = radius.dp.toPx()
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

		// State label — always takes space to keep the circle position fixed
		Column(
			horizontalAlignment = Alignment.CenterHorizontally,
			modifier = Modifier.alpha(labelAlpha),
		) {
			Spacer(Modifier.height(LABEL_OFFSET.dp))
			val isOffline = connectionState == ConnectionState.Offline ||
				connectionState == ConnectionState.WaitingForConnection
			Row(
				verticalAlignment = Alignment.CenterVertically,
				horizontalArrangement = Arrangement.Center,
			) {
				if (isOffline) {
					Pulse(color = MaterialTheme.colorScheme.error)
					Spacer(Modifier.width(6.dp))
				}
				Text(
					text = (currentLabel ?: "").uppercase(),
					style = MaterialTheme.typography.labelSmall,
					color = when {
						isError -> MaterialTheme.colorScheme.error
						isConnected -> MaterialTheme.colorScheme.primary
						isOffline -> MaterialTheme.colorScheme.error
						else -> MaterialTheme.colorScheme.onSurfaceVariant
					},
				)
			}
			Spacer(Modifier.height(8.dp))
			Text(
				text = connectionTime ?: "",
				style = MaterialTheme.typography.labelSmall,
				color = MaterialTheme.colorScheme.primary,
				modifier = Modifier.alpha(timeAlpha),
			)
		}
	}
}

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

private fun ringFillTargets(connectionState: ConnectionState, establishConnectionState: EstablishConnectionState?): RingFillTargets = when (connectionState) {
	ConnectionState.Connected -> RingFillTargets(360f, 360f, 360f)
	is ConnectionState.Error, is ConnectionState.StartFailure -> RingFillTargets(360f, 360f, 0f)
	ConnectionState.Disconnecting -> RingFillTargets(0f, 0f, 0f)
	is ConnectionState.Connecting -> when (establishConnectionState) {
		EstablishConnectionState.AWAITING_ACCOUNT_READINESS -> RingFillTargets(360f, 0f, 0f)
		EstablishConnectionState.REFRESHING_GATEWAYS -> RingFillTargets(360f, 180f, 0f)
		EstablishConnectionState.SELECTING_GATEWAYS -> RingFillTargets(360f, 360f, 0f)
		EstablishConnectionState.REGISTERING_WITH_GATEWAYS -> RingFillTargets(360f, 360f, 180f)
		EstablishConnectionState.CONNECTING_TUNNEL -> RingFillTargets(360f, 360f, 360f)
		else -> RingFillTargets(180f, 0f, 0f)
	}
	else -> RingFillTargets(0f, 0f, 0f)
}

@Composable
@Preview(showBackground = true)
private fun ArcPreviewConnectedDark() {
	NymVPNTheme(Theme.DARK_MODE) {
		Surface(modifier = Modifier.background(MaterialTheme.colorScheme.background)) {
			ConnectionStatus(
				connectionState = ConnectionState.Disconnected,
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
			connectionState = ConnectionState.Error(nym_vpn_lib_types.ErrorStateReason.InactiveSubscription),
			vpnMode = Tunnel.Mode.TWO_HOP_MIXNET,
		)
	}
}
