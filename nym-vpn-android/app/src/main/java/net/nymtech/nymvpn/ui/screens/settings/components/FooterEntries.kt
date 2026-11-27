package net.nymtech.nymvpn.ui.screens.settings.components

import android.content.res.Configuration
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.outlined.Logout
import androidx.compose.material.icons.outlined.Close
import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.common.buttons.surface.SelectionItem
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme

@Composable
fun footerEntries(isMnemonicStored: Boolean, appVersion: String, onLogoutClick: () -> Unit, onQuitClick: () -> Unit, onAppVersionClick: () -> Unit): List<SettingsEntry> = buildList {
	if (isMnemonicStored) {
		add(
			SettingsBlock(
				section = SettingsSection.FOOTER,
				title = stringResource(R.string.log_out),
				content = {
					SettingsGroup(
						items = listOf(
							SelectionItem(
								leading = {
									SettingsIcon(
										Icons.AutoMirrored.Outlined.Logout,
										stringResource(R.string.log_out),
									)
								},
								title = {
									SettingsTitle(stringResource(R.string.log_out))
								},
								onClick = onLogoutClick,
							),
						),
					)
				},
			),
		)
	}

	add(
		SettingsRow(
			section = SettingsSection.FOOTER,
			title = stringResource(R.string.settings_quit_title),
			item = SelectionItem(
				leading = {
					SettingsIcon(
						Icons.Outlined.Close,
						stringResource(R.string.settings_quit_title),
					)
				},
				title = {
					SettingsTitle(stringResource(R.string.settings_quit_title))
				},
				onClick = onQuitClick,
			),
		),
	)

	add(
		SettingsBlock(
			section = SettingsSection.FOOTER,
			title = stringResource(R.string.settings_app_version_line, appVersion),
			content = { AppVersionText(appVersion = appVersion, onAppVersionClick = onAppVersionClick) },
		),
	)
}

@Composable
@Preview(uiMode = Configuration.UI_MODE_NIGHT_YES)
internal fun PreviewFooterEntries() {
	NymVPNTheme(Theme.default()) {
		SettingsEntryList(
			entries = footerEntries(isMnemonicStored = true, appVersion = "2.2.2", onLogoutClick = {}, onQuitClick = {}, onAppVersionClick = {}),
		)
	}
}
