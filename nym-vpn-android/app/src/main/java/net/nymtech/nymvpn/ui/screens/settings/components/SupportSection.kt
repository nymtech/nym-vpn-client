package net.nymtech.nymvpn.ui.screens.settings.components

import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.res.vectorResource
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.common.buttons.surface.SelectionItem

@Composable
fun SupportSection(onSupportClick: () -> Unit) {
	SettingsGroup(
		items = listOf(
			SelectionItem(
				leading = {
					SettingsIcon(
						ImageVector.vectorResource(R.drawable.support),
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
}
