package net.nymtech.nymvpn.ui.screens.main.components

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Info
import androidx.compose.material.icons.outlined.Speed
import androidx.compose.material.icons.outlined.VisibilityOff
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.common.buttons.IconSurfaceButton
import net.nymtech.nymvpn.ui.common.labels.GroupLabel
import net.nymtech.nymvpn.ui.common.snackbar.SnackbarController
import net.nymtech.nymvpn.ui.model.ConnectionState
import net.nymtech.nymvpn.ui.theme.iconSize
import net.nymtech.nymvpn.util.extensions.scaledHeight
import net.nymtech.nymvpn.util.extensions.scaledWidth
import net.nymtech.vpn.backend.Tunnel

@Composable
fun ModeSelector(
	vpnMode: Tunnel.Mode,
	connectionState: ConnectionState,
	onTwoHopClick: () -> Unit,
	onFiveHopClick: () -> Unit,
	onInfoClick: () -> Unit,
	modifier: Modifier = Modifier,
	snackbar: SnackbarController,
) {
	val context = LocalContext.current

	fun whenDisconnected(callback: () -> Unit) {
		when (connectionState) {
			ConnectionState.Disconnected, ConnectionState.Offline -> callback.invoke()
			ConnectionState.WaitingForConnection, is ConnectionState.Connecting -> snackbar.showMessage(context.getString(R.string.disabled_while_connecting))
			else -> snackbar.showMessage(context.getString(R.string.disabled_while_connected))
		}
	}

	Column(
		verticalArrangement = Arrangement.spacedBy(24.dp.scaledHeight(), Alignment.Bottom),
		modifier = modifier.padding(horizontal = 24.dp.scaledWidth()),
	) {
		Row(
			horizontalArrangement = Arrangement.SpaceBetween,
			verticalAlignment = Alignment.CenterVertically,
			modifier = Modifier.fillMaxWidth().padding(bottom = 16.dp.scaledHeight()),
		) {
			GroupLabel(title = stringResource(R.string.select_mode))
			IconButton(onClick = onInfoClick, modifier = Modifier.size(iconSize)) {
				Icon(Icons.Outlined.Info, stringResource(R.string.info), tint = MaterialTheme.colorScheme.outline)
			}
		}
		IconSurfaceButton(
			leading = {
				Icon(
					Icons.Outlined.Speed,
					contentDescription = stringResource(R.string.fastest),
					Modifier.size(iconSize.scaledWidth()),
					tint = if (vpnMode == Tunnel.Mode.TWO_HOP_MIXNET) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.onSurface,
				)
			},
			title = stringResource(R.string.two_hop_title),
			description = stringResource(R.string.two_hop_description),
			onClick = { whenDisconnected(onTwoHopClick) },
			selected = vpnMode == Tunnel.Mode.TWO_HOP_MIXNET,
		)
		IconSurfaceButton(
			leading = {
				Icon(
					Icons.Outlined.VisibilityOff,
					contentDescription = stringResource(R.string.anonymous),
					Modifier.size(iconSize.scaledWidth()),
					tint = if (vpnMode == Tunnel.Mode.FIVE_HOP_MIXNET) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.onSurface,
				)
			},
			title = stringResource(R.string.five_hop_mixnet),
			description = stringResource(R.string.five_hop_description),
			onClick = { whenDisconnected(onFiveHopClick) },
			selected = vpnMode == Tunnel.Mode.FIVE_HOP_MIXNET,
		)
	}
}
