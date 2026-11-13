package net.nymtech.nymvpn.ui.screens.main.bottomsheet.auth.components

import android.content.res.Configuration
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Analytics
import androidx.compose.material.icons.outlined.BugReport
import androidx.compose.material.icons.outlined.Info
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.common.TelemetryInfoKind
import net.nymtech.nymvpn.ui.common.buttons.ScaledSwitch
import net.nymtech.nymvpn.ui.common.buttons.surface.SelectionItem
import net.nymtech.nymvpn.ui.screens.settings.components.SettingsGroup
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.ui.theme.iconSize
import net.nymtech.nymvpn.util.extensions.scaledWidth

@Composable
fun SettingsSection(
	statsEnabled: Boolean,
	sentryEnabled: Boolean,
	onNetworkStatsEnable: (enabled: Boolean) -> Unit,
	onMonitoringEnable: (enabled: Boolean) -> Unit,
	onInfo: (TelemetryInfoKind) -> Unit,
) {
	SettingsGroup(
		items = listOf(
			SelectionItem(
				leading = {
					Icon(
						Icons.Outlined.Analytics,
						stringResource(R.string.optin_connection_title),
						modifier = Modifier.size(iconSize.scaledWidth()),
					)
				},
				trailing = {
					ScaledSwitch(
						checked = statsEnabled,
						onClick = { onNetworkStatsEnable(it) },
					)
				},
				title = {
					Row(verticalAlignment = Alignment.CenterVertically) {
						Text(
							stringResource(R.string.optin_connection_title),
							style = MaterialTheme.typography.bodyMedium,
							color = MaterialTheme.colorScheme.onPrimaryContainer,
							fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
						)
						IconButton(
							onClick = { onInfo(TelemetryInfoKind.CONNECTION) },
							modifier = Modifier.size(24.dp),
						) {
							Icon(
								Icons.Outlined.Info,
								contentDescription = stringResource(R.string.optin_info_accessibility, stringResource(R.string.optin_connection_title)),
								tint = MaterialTheme.colorScheme.onBackground,
							)
						}
					}
				},
				description = {
					Text(
						text = stringResource(R.string.optin_connection_subtitle),
						style = MaterialTheme.typography.bodySmall,
						color = MaterialTheme.colorScheme.onBackground,
						fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
					)
				},
			),
			SelectionItem(
				leading = {
					Icon(
						Icons.Outlined.BugReport,
						stringResource(R.string.optin_monitoring_title),
						modifier = Modifier.size(iconSize.scaledWidth()),
					)
				},
				trailing = {
					ScaledSwitch(
						checked = sentryEnabled,
						onClick = { onMonitoringEnable(it) },
					)
				},
				title = {
					Row(verticalAlignment = Alignment.CenterVertically) {
						Text(
							stringResource(R.string.optin_monitoring_title),
							style = MaterialTheme.typography.bodyMedium,
							color = MaterialTheme.colorScheme.onPrimaryContainer,
							fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
						)
						IconButton(
							onClick = { onInfo(TelemetryInfoKind.MONITORING) },
							modifier = Modifier.size(24.dp),
						) {
							Icon(
								Icons.Outlined.Info,
								contentDescription = stringResource(R.string.optin_info_accessibility, stringResource(R.string.optin_monitoring_title)),
								tint = MaterialTheme.colorScheme.onBackground,
							)
						}
					}
				},
				description = {
					Text(
						text = stringResource(R.string.optin_monitoring_subtitle),
						style = MaterialTheme.typography.bodySmall,
						color = MaterialTheme.colorScheme.onBackground,
						fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
					)
				},
			),
		),
	)
}

@Preview(uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
internal fun PreviewSettingsSection() {
	NymVPNTheme(Theme.default()) {
		SettingsSection(
			statsEnabled = true,
			sentryEnabled = true,
			onNetworkStatsEnable = {},
			onMonitoringEnable = {},
			onInfo = {},
		)
	}
}
