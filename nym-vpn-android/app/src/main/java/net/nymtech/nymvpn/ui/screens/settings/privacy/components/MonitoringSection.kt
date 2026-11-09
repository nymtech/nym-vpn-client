package net.nymtech.nymvpn.ui.screens.settings.privacy.components

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Info
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Shape
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.common.TelemetryInfoKind
import net.nymtech.nymvpn.ui.common.buttons.ScaledSwitch
import net.nymtech.nymvpn.ui.theme.LocalNymColors

@Composable
fun MonitoringSection(
	sentryEnabled: Boolean,
	onMonitoringEnable: (enabled: Boolean) -> Unit,
	onInfo: (TelemetryInfoKind) -> Unit,
	shape: Shape = RoundedCornerShape(8.dp),
) {
	Card(
		modifier = Modifier.fillMaxWidth(),
		shape = shape,
		colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.primaryContainer),
	) {
		Row(
			modifier = Modifier
				.fillMaxWidth()
				.padding(horizontal = 16.dp, vertical = 16.dp),
			horizontalArrangement = Arrangement.SpaceBetween,
			verticalAlignment = Alignment.CenterVertically,
		) {
			Column(modifier = Modifier.weight(1f)) {
				Row(verticalAlignment = Alignment.CenterVertically) {
					Text(
						text = stringResource(R.string.optin_monitoring_title),
						style = MaterialTheme.typography.titleMedium,
						color = MaterialTheme.colorScheme.onPrimaryContainer,
					)
					IconButton(
						onClick = { onInfo(TelemetryInfoKind.MONITORING) },
						modifier = Modifier.size(24.dp),
					) {
						Icon(
							Icons.Outlined.Info,
							contentDescription =
								stringResource(
									R.string.optin_info_accessibility,
									stringResource(R.string.optin_monitoring_title),
								),
							tint = MaterialTheme.colorScheme.onBackground,
						)
					}
				}
				Text(
					text = stringResource(R.string.optin_monitoring_subtitle),
					style = MaterialTheme.typography.bodySmall,
					color = MaterialTheme.colorScheme.onBackground,
				)
				Text(
					text = stringResource(R.string.privacy_error_reports_restart),
					style = MaterialTheme.typography.bodySmall,
					color = LocalNymColors.current.warning,
				)
			}
			ScaledSwitch(
				checked = sentryEnabled,
				onClick = { onMonitoringEnable(it) },
			)
		}
	}
}
