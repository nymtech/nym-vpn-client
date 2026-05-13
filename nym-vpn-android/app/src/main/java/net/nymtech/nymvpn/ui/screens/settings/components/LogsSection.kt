package net.nymtech.nymvpn.ui.screens.settings.components

import android.content.res.Configuration
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Lock
import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.common.buttons.surface.SelectionItem
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme

@Composable
fun LogsSection(onPrivacyClick: () -> Unit) {
	SettingsGroup(
		items = listOf(
			SelectionItem(
				leading = {
					SettingsIcon(
						Icons.Outlined.Lock,
						stringResource(R.string.settings_logs_title),
					)
				},
				trailing = {
					SettingsArrowIcon()
				},
				title = {
					SettingsTitle(stringResource(R.string.settings_logs_title))
				},
				onClick = onPrivacyClick,
			),
		),
	)
}

@Composable
@Preview(uiMode = Configuration.UI_MODE_NIGHT_YES)
internal fun PreviewLogsSection() {
	NymVPNTheme(Theme.default()) {
		LogsSection({})
	}
}
