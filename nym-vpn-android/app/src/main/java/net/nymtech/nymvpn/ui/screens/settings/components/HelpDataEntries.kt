package net.nymtech.nymvpn.ui.screens.settings.components

import android.content.res.Configuration
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.outlined.InsertDriveFile
import androidx.compose.material.icons.outlined.ChatBubbleOutline
import androidx.compose.material.icons.outlined.Lock
import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.common.buttons.surface.SelectionItem
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme

@Composable
fun helpDataEntries(onSupportClick: () -> Unit, onPrivacyClick: () -> Unit, onLegalClick: () -> Unit): List<SettingsEntry> = buildList {
	add(
		SettingsRow(
			section = SettingsSection.HELP_DATA,
			title = stringResource(R.string.support_and_feedback),
			item = SelectionItem(
				leading = {
					SettingsIcon(
						Icons.Outlined.ChatBubbleOutline,
						stringResource(R.string.support),
					)
				},
				trailing = {
					SettingsArrowIcon()
				},
				title = {
					SettingsTitle(stringResource(R.string.support_and_feedback))
				},
				onClick = onSupportClick,
			),
		),
	)

	add(
		SettingsRow(
			section = SettingsSection.HELP_DATA,
			title = stringResource(R.string.settings_logs_title),
			item = SelectionItem(
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

	add(
		SettingsRow(
			section = SettingsSection.HELP_DATA,
			title = stringResource(R.string.settings_legal_title),
			item = SelectionItem(
				leading = {
					SettingsIcon(
						Icons.AutoMirrored.Outlined.InsertDriveFile,
						stringResource(R.string.settings_legal_title),
					)
				},
				trailing = {
					SettingsArrowIcon()
				},
				title = {
					SettingsTitle(stringResource(R.string.settings_legal_title))
				},
				onClick = onLegalClick,
			),
		),
	)
}

@Composable
@Preview(uiMode = Configuration.UI_MODE_NIGHT_YES)
internal fun PreviewHelpDataEntries() {
	NymVPNTheme(Theme.default()) {
		SettingsEntryList(entries = helpDataEntries(onSupportClick = {}, onPrivacyClick = {}, onLegalClick = {}))
	}
}
