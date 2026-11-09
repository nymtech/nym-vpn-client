package net.nymtech.nymvpn.ui.screens.settings.privacy.components

import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.Shape
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.common.TelemetryInfoKind

@Composable
fun MonitoringSection(sentryEnabled: Boolean, onMonitoringEnable: (enabled: Boolean) -> Unit, onInfo: (TelemetryInfoKind) -> Unit, shape: Shape = RoundedCornerShape(8.dp)) {
	TelemetryToggleCard(
		title = stringResource(R.string.optin_monitoring_title),
		subtitle = stringResource(R.string.optin_monitoring_subtitle),
		kind = TelemetryInfoKind.MONITORING,
		checked = sentryEnabled,
		onCheckedChange = onMonitoringEnable,
		onInfo = onInfo,
		shape = shape,
		warning = stringResource(R.string.privacy_error_reports_restart),
	)
}
