package net.nymtech.nymvpn.ui.screens.settings.components

import android.content.res.Configuration
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.outlined.InsertDriveFile
import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.common.buttons.surface.SelectionItem
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme

@Composable
fun legalEntries(onLegalClick: () -> Unit): List<SettingsEntry> = buildList {
	add(
		R.string.settings_section_help_data,
		stringResource(R.string.settings_legal_title),
		SelectionItem(
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
	)
}

@Composable
@Preview(uiMode = Configuration.UI_MODE_NIGHT_YES)
internal fun PreviewLegalSection() {
	NymVPNTheme(Theme.default()) {
		SettingsGroup(items = legalEntries {}.map { it.item })
	}
}
