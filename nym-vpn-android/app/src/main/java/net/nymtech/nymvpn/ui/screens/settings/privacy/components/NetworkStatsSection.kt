package net.nymtech.nymvpn.ui.screens.settings.privacy.components

import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Shape
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.common.TelemetryInfoKind

@Composable
fun NetworkStatsSection(
	statsEnabled: Boolean,
	onNetworkStatsEnable: (enabled: Boolean) -> Unit,
	onInfo: (TelemetryInfoKind) -> Unit,
	shape: Shape = RoundedCornerShape(8.dp),
) {
	TelemetryToggleCard(
		title = stringResource(R.string.optin_connection_title),
		subtitle = stringResource(R.string.optin_connection_subtitle),
		kind = TelemetryInfoKind.CONNECTION,
		checked = statsEnabled,
		onCheckedChange = onNetworkStatsEnable,
		onInfo = onInfo,
		shape = shape,
	)
}
