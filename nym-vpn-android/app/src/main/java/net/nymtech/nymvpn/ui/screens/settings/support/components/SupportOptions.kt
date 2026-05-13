package net.nymtech.nymvpn.ui.screens.settings.support.components

import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Language
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.res.vectorResource
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.common.buttons.surface.SelectionItem
import net.nymtech.nymvpn.ui.screens.settings.components.SettingsArrowIcon
import net.nymtech.nymvpn.ui.screens.settings.components.SettingsGroup
import net.nymtech.nymvpn.ui.screens.settings.components.SettingsIcon
import net.nymtech.nymvpn.ui.screens.settings.components.SettingsTitle
import net.nymtech.nymvpn.util.extensions.openWebUrl

@Composable
fun SupportOptions() {
	val context = LocalContext.current
	SettingsGroup(
		items = listOf(
			SelectionItem(
				leading = {
					SettingsIcon(
						ImageVector.vectorResource(R.drawable.faq),
						stringResource(R.string.check_faq),
					)
				},
				trailing = {
					SettingsArrowIcon()
				},
				title = {
					SettingsTitle(
						stringResource(R.string.check_faq),
					)
				},
				onClick = { context.openWebUrl(context.getString(R.string.faq_url)) },
			),
			SelectionItem(
				leading = {
					SettingsIcon(
						ImageVector.vectorResource(R.drawable.send),
						stringResource(R.string.get_help),
					)
				},
				trailing = {
					SettingsArrowIcon()
				},
				title = {
					SettingsTitle(
						stringResource(R.string.get_in_touch),
					)
				},
				onClick = { context.openWebUrl(context.getString(R.string.contact_url)) },
			),
		),
	)
	SettingsGroup(
		items = listOf(
			SelectionItem(
				leading = {
					SettingsIcon(
						ImageVector.vectorResource(R.drawable.github),
						stringResource(R.string.github_issues_url),
					)
				},
				trailing = {
					SettingsArrowIcon()
				},
				title = {
					SettingsTitle(
						stringResource(R.string.open_github),
					)
				},
				onClick = { context.openWebUrl(context.getString(R.string.github_issues_url)) },
			),

			SelectionItem(
				leading = {
					SettingsIcon(
						ImageVector.vectorResource(R.drawable.telegram),
						stringResource(R.string.telegram_url),
					)
				},
				trailing = {
					SettingsArrowIcon()
				},
				title = {
					SettingsTitle(
						stringResource(R.string.join_telegram),
					)
				},
				onClick = { context.openWebUrl(context.getString(R.string.telegram_url)) },
			),

			SelectionItem(
				leading = {
					SettingsIcon(
						ImageVector.vectorResource(R.drawable.matrix),
						stringResource(R.string.matrix_url),
					)
				},
				trailing = {
					SettingsArrowIcon()
				},
				title = {
					SettingsTitle(
						stringResource(R.string.join_matrix),
					)
				},
				onClick = { context.openWebUrl(context.getString(R.string.matrix_url)) },
			),

			SelectionItem(
				leading = {
					SettingsIcon(
						ImageVector.vectorResource(R.drawable.discord),
						stringResource(R.string.join_discord),
					)
				},
				trailing = {
					SettingsArrowIcon()
				},
				title = {
					SettingsTitle(
						stringResource(R.string.join_discord),
					)
				},
				onClick = { context.openWebUrl(context.getString(R.string.discord_url)) },
			),
		),
	)
	SettingsGroup(
		items = listOf(
			SelectionItem(
				leading = {
					SettingsIcon(
						Icons.Outlined.Language,
						stringResource(R.string.settings_language_help_title),
					)
				},
				trailing = {
					SettingsArrowIcon()
				},
				title = {
					SettingsTitle(
						stringResource(R.string.settings_language_help_title),
					)
				},
				description = {
					Text(
						stringResource(R.string.settings_language_help_description),
						style = MaterialTheme.typography.bodySmall.copy(MaterialTheme.colorScheme.onBackground),
					)
				},
				onClick = { context.openWebUrl(context.getString(R.string.discord_url)) },
			),
		),
	)
}
