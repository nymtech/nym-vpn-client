package net.nymtech.nymvpn.ui.screens.main.bottomsheet.auth.components

import android.content.res.Configuration
import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Analytics
import androidx.compose.material.icons.outlined.BugReport
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.LinkAnnotation
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.style.TextDecoration
import androidx.compose.ui.text.withLink
import androidx.compose.ui.text.withStyle
import androidx.compose.ui.tooling.preview.Preview
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.common.buttons.ScaledSwitch
import net.nymtech.nymvpn.ui.common.buttons.surface.SelectionItem
import net.nymtech.nymvpn.ui.screens.settings.components.SettingsGroup
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.ui.theme.iconSize
import net.nymtech.nymvpn.util.extensions.scaledWidth

@Composable
fun SettingsSection(statsEnabled: Boolean, sentryEnabled: Boolean, onNetworkStatsEnable: (enabled: Boolean) -> Unit, onMonitoringEnable: (enabled: Boolean) -> Unit) {
	SettingsGroup(
		items = listOf(
			SelectionItem(
				leading = {
					Icon(
						Icons.Outlined.BugReport,
						stringResource(R.string.welcome_error_reports_title),
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
					Text(
						stringResource(R.string.welcome_error_reports_title),
						style = MaterialTheme.typography.bodyMedium,
						color = MaterialTheme.colorScheme.onPrimaryContainer,
						fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
					)
				},
				description = {
					Text(
						text = buildAnnotatedString {
							append(stringResource(R.string.welcome_error_reports_description_start))
							append(" ")
							withStyle(
								style = SpanStyle(
									color = MaterialTheme.colorScheme.primary,
									textDecoration = TextDecoration.None,
								),
							) {
								withLink(LinkAnnotation.Url(stringResource(R.string.welcome_error_reports_link))) {
									append(stringResource(R.string.welcome_error_reports_link_text))
								}
							}
							append(stringResource(R.string.welcome_error_reports_description_end))
						},
						style = MaterialTheme.typography.bodySmall,
						color = MaterialTheme.colorScheme.onBackground,
						fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
					)
				},
			),
			SelectionItem(
				leading = {
					Icon(
						Icons.Outlined.Analytics,
						stringResource(R.string.welcome_usage_title),
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
					Text(
						stringResource(R.string.welcome_usage_title),
						style = MaterialTheme.typography.bodyMedium,
						color = MaterialTheme.colorScheme.onPrimaryContainer,
						fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
					)
				},
				description = {
					Text(
						text = stringResource(R.string.welcome_usage_description),
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
		)
	}
}
