package net.nymtech.nymvpn.ui.screens.settings.components

import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.ChatBubbleOutline
import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.common.buttons.surface.SelectionItem

@Composable
fun supportEntries(onSupportClick: () -> Unit): List<SettingsEntry> = buildList {
	add(
		R.string.settings_section_help_data,
		stringResource(R.string.support_and_feedback),
		SelectionItem(
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
	)
}
