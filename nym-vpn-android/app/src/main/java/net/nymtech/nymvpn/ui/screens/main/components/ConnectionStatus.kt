package net.nymtech.nymvpn.ui.screens.main.components

import android.content.Context
import android.provider.Settings
import androidx.compose.animation.core.FastOutSlowInEasing
import androidx.compose.animation.core.animateFloatAsState
import androidx.compose.animation.core.tween
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.size
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Surface
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.graphicsLayer
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.lifecycle.compose.LifecycleResumeEffect
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.model.ConnectionState
import net.nymtech.nymvpn.ui.theme.LocalNymColors
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.util.StringValue
import net.nymtech.vpn.backend.Tunnel
import nym_vpn_lib_types.ErrorStateReason

@Composable
fun ConnectionStatus(connectionState: ConnectionState, lockdown: Boolean, vpnMode: Tunnel.Mode, connectionTime: String? = null, modifier: Modifier = Modifier) {
	val context = LocalContext.current
	val resumes = rememberResumeCount()
	val animationsOff = remember(context, resumes) { areAnimationsOff(context) }
	// blocked: a live session waiting for the network while the system holds traffic
	val isBlocked = connectionState == ConnectionState.WaitingForConnection && lockdown
	val isUnprotected = connectionState == ConnectionState.Disconnected ||
		connectionState == ConnectionState.Offline ||
		(connectionState == ConnectionState.WaitingForConnection && !isBlocked)

	Column(horizontalAlignment = Alignment.CenterHorizontally, modifier = modifier) {
		Box(contentAlignment = Alignment.Center) {
			ConnectionRings(connectionState = connectionState, isBlocked = isBlocked, vpnMode = vpnMode, animationsOff = animationsOff)
			MarkerFace(
				vpnMode = vpnMode,
				lit = !isUnprotected,
				playing = connectionState == ConnectionState.Connected && !animationsOff,
				litColor = MaterialTheme.colorScheme.onPrimaryContainer,
				unlitColor = LocalNymColors.current.markerTrack,
				modifier = Modifier.size(FACE_SIZE.dp),
			)
		}
		ConnectionStatusText(connectionState = connectionState, isBlocked = isBlocked, vpnMode = vpnMode, connectionTime = connectionTime)
	}
}

// both lines always take space, so the rings never move
@Composable
private fun ConnectionStatusText(connectionState: ConnectionState, isBlocked: Boolean, vpnMode: Tunnel.Mode, connectionTime: String?) {
	val context = LocalContext.current
	val notProtectedLabel = stringResource(R.string.connection_status_not_protected)
	val label = when (connectionState) {
		ConnectionState.Connected -> stringResource(if (vpnMode.isTwoHop()) R.string.connection_status_fast_mode else R.string.connection_status_anonymous)
		is ConnectionState.Connecting -> connectionState.label.asString(context)
		is ConnectionState.Error, is ConnectionState.StartFailure -> stringResource(R.string.connection_failed)
		ConnectionState.Disconnecting -> stringResource(R.string.disconnecting)
		ConnectionState.WaitingForConnection -> if (isBlocked) stringResource(R.string.connection_status_blocked) else notProtectedLabel
		ConnectionState.Disconnected, ConnectionState.Offline -> notProtectedLabel
	}
	val offlineCaption = when (connectionState) {
		ConnectionState.WaitingForConnection -> stringResource(if (isBlocked) R.string.connection_status_blocked_action else R.string.connection_status_offline_reconnect)
		ConnectionState.Offline -> stringResource(R.string.connection_status_no_internet)
		else -> null
	}
	val caption = offlineCaption ?: connectionTime
	val captionAlpha by animateFloatAsState(
		targetValue = if (caption != null) 1f else 0f,
		animationSpec = tween(250, easing = FastOutSlowInEasing),
		label = "captionAlpha",
	)

	Column(horizontalAlignment = Alignment.CenterHorizontally) {
		Spacer(Modifier.height(14.dp))
		Text(
			text = label,
			style = if (connectionState is ConnectionState.Connecting) {
				MaterialTheme.typography.labelSmall.copy(fontWeight = FontWeight.Normal)
			} else {
				MaterialTheme.typography.labelSmall
			},
			color = when {
				connectionState.isFailure -> LocalNymColors.current.markerError
				connectionState == ConnectionState.Connected -> MaterialTheme.colorScheme.primary
				else -> MaterialTheme.colorScheme.onSurfaceVariant
			},
		)
		Spacer(Modifier.height(8.dp))
		Text(
			text = caption ?: "",
			style = MaterialTheme.typography.labelSmall,
			color = if (offlineCaption != null) MaterialTheme.colorScheme.onSurfaceVariant else MaterialTheme.colorScheme.primary,
			modifier = Modifier.graphicsLayer { alpha = captionAlpha },
		)
	}
}

/** Counts lifecycle resumes, to key values read from system settings. */
@Composable
private fun rememberResumeCount(): Int {
	var resumes by remember { mutableIntStateOf(0) }
	LifecycleResumeEffect(Unit) {
		resumes++
		onPauseOrDispose { }
	}
	return resumes
}

private fun areAnimationsOff(context: Context): Boolean = Settings.Global.getFloat(context.contentResolver, Settings.Global.ANIMATOR_DURATION_SCALE, 1f) == 0f

@Composable
@Preview(showBackground = true)
private fun ConnectionStatusNotProtectedPreview() {
	NymVPNTheme(Theme.DARK_MODE) {
		Surface(modifier = Modifier.background(MaterialTheme.colorScheme.background)) {
			ConnectionStatus(connectionState = ConnectionState.Disconnected, lockdown = false, vpnMode = Tunnel.Mode.TWO_HOP_MIXNET)
		}
	}
}

@Composable
@Preview(showBackground = true)
private fun ConnectionStatusConnectedPreview() {
	NymVPNTheme(Theme.LIGHT_MODE) {
		Surface(modifier = Modifier.background(MaterialTheme.colorScheme.background)) {
			ConnectionStatus(connectionState = ConnectionState.Connected, lockdown = false, vpnMode = Tunnel.Mode.TWO_HOP_MIXNET, connectionTime = "00:12:34")
		}
	}
}

@Composable
@Preview(showBackground = true)
private fun ConnectionStatusErrorPreview() {
	NymVPNTheme(Theme.DARK_MODE) {
		ConnectionStatus(connectionState = ConnectionState.Error(ErrorStateReason.InactiveSubscription), lockdown = false, vpnMode = Tunnel.Mode.TWO_HOP_MIXNET)
	}
}

@Composable
@Preview(showBackground = true)
private fun ConnectionStatusConnectingPreview() {
	NymVPNTheme(Theme.DARK_MODE) {
		ConnectionStatus(
			connectionState = ConnectionState.Connecting(StringValue.DynamicString("Choosing best servers"), step = 4),
			lockdown = false,
			vpnMode = Tunnel.Mode.FIVE_HOP_MIXNET,
		)
	}
}

@Composable
@Preview(showBackground = true)
private fun ConnectionStatusBlockedPreview() {
	NymVPNTheme(Theme.LIGHT_MODE) {
		ConnectionStatus(connectionState = ConnectionState.WaitingForConnection, lockdown = true, vpnMode = Tunnel.Mode.FIVE_HOP_MIXNET)
	}
}
