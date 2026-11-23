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

/**
 * Shared card for a telemetry opt-in toggle on the Data & privacy screen:
 * title + info button, subtitle, an optional warning line, and a switch. Used by
 * the connection (network stats) and monitoring (crash reports) sections so they
 * don't each duplicate the layout.
 */
@Composable
fun TelemetryToggleCard(
	title: String,
	subtitle: String,
	kind: TelemetryInfoKind,
	checked: Boolean,
	onCheckedChange: (enabled: Boolean) -> Unit,
	onInfo: (TelemetryInfoKind) -> Unit,
	shape: Shape = RoundedCornerShape(8.dp),
	warning: String? = null,
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
						text = title,
						modifier = Modifier.weight(1f, fill = false),
						style = MaterialTheme.typography.titleMedium,
						color = MaterialTheme.colorScheme.onPrimaryContainer,
					)
					IconButton(
						onClick = { onInfo(kind) },
						modifier = Modifier.size(24.dp),
					) {
						Icon(
							Icons.Outlined.Info,
							contentDescription = stringResource(R.string.optin_info_accessibility, title),
							tint = MaterialTheme.colorScheme.onBackground,
						)
					}
				}
				Text(
					text = subtitle,
					style = MaterialTheme.typography.bodySmall,
					color = MaterialTheme.colorScheme.onBackground,
				)
				if (warning != null) {
					Text(
						text = warning,
						style = MaterialTheme.typography.bodySmall,
						color = LocalNymColors.current.warning,
					)
				}
			}
			ScaledSwitch(
				checked = checked,
				onClick = { onCheckedChange(it) },
			)
		}
	}
}
