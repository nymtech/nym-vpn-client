package net.nymtech.nymvpn.ui.screens.settings.support.components

import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.outlined.ArrowRight
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.res.vectorResource
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.common.buttons.surface.SelectionItem
import net.nymtech.nymvpn.ui.screens.settings.components.SettingsGroup
import net.nymtech.nymvpn.ui.theme.iconSize
import net.nymtech.nymvpn.util.extensions.openWebUrl
import net.nymtech.nymvpn.util.extensions.scaledWidth

@Composable
fun SupportOptions() {
	val context = LocalContext.current
	SettingsGroup(
		items = listOf(
			SelectionItem(
				leading = {
					Icon(
						ImageVector.vectorResource(R.drawable.faq),
						stringResource(R.string.check_faq),
						modifier = Modifier.size(iconSize.scaledWidth()),
					)
				},
				trailing = {
					Icon(
						Icons.AutoMirrored.Outlined.ArrowRight,
						stringResource(R.string.go),
						modifier = Modifier.size(iconSize),
					)
				},
				title = {
					Text(
						stringResource(R.string.check_faq),
						style = MaterialTheme.typography.bodyLarge.copy(MaterialTheme.colorScheme.onSurface),
					)
				},
				onClick = { context.openWebUrl(context.getString(R.string.faq_url)) },
			),
		),
	)
	SettingsGroup(
		items = listOf(
			SelectionItem(
				leading = {
					Icon(
						ImageVector.vectorResource(R.drawable.send),
						stringResource(R.string.get_help),
						modifier = Modifier.size(iconSize.scaledWidth()),
					)
				},
				trailing = {
					Icon(
						Icons.AutoMirrored.Outlined.ArrowRight,
						stringResource(R.string.go),
						modifier = Modifier.size(iconSize),
					)
				},
				title = {
					Text(
						stringResource(R.string.get_in_touch),
						style = MaterialTheme.typography.bodyLarge.copy(MaterialTheme.colorScheme.onSurface),
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
					Icon(
						ImageVector.vectorResource(R.drawable.github),
						stringResource(R.string.github_issues_url),
						modifier = Modifier.size(iconSize.scaledWidth()),
					)
				},
				trailing = {
					Icon(
						Icons.AutoMirrored.Outlined.ArrowRight,
						stringResource(R.string.go),
						modifier = Modifier.size(iconSize),
					)
				},
				title = {
					Text(
						stringResource(R.string.open_github),
						style = MaterialTheme.typography.bodyLarge.copy(MaterialTheme.colorScheme.onSurface),
					)
				},
				onClick = { context.openWebUrl(context.getString(R.string.github_issues_url)) },
			),
		),
	)
	SettingsGroup(
		items = listOf(
			SelectionItem(
				leading = {
					Icon(
						ImageVector.vectorResource(R.drawable.telegram),
						stringResource(R.string.telegram_url),
						modifier = Modifier.size(iconSize.scaledWidth()),
					)
				},
				trailing = {
					Icon(
						Icons.AutoMirrored.Outlined.ArrowRight,
						stringResource(R.string.go),
						modifier = Modifier.size(iconSize),
					)
				},
				title = {
					Text(
						stringResource(R.string.join_telegram),
						style = MaterialTheme.typography.bodyLarge.copy(MaterialTheme.colorScheme.onSurface),
					)
				},
				onClick = { context.openWebUrl(context.getString(R.string.telegram_url)) },
			),
		),
	)
	SettingsGroup(
		items = listOf(
			SelectionItem(
				leading = {
					Icon(
						ImageVector.vectorResource(R.drawable.matrix),
						stringResource(R.string.matrix_url),
						modifier = Modifier.size(iconSize.scaledWidth()),
					)
				},
				trailing = {
					Icon(
						Icons.AutoMirrored.Outlined.ArrowRight,
						stringResource(R.string.go),
						modifier = Modifier.size(iconSize),
					)
				},
				title = {
					Text(
						stringResource(R.string.join_matrix),
						style = MaterialTheme.typography.bodyLarge.copy(MaterialTheme.colorScheme.onSurface),
					)
				},
				onClick = { context.openWebUrl(context.getString(R.string.matrix_url)) },
			),
		),
	)
	SettingsGroup(
		items = listOf(
			SelectionItem(
				leading = {
					Icon(
						ImageVector.vectorResource(R.drawable.discord),
						stringResource(R.string.join_discord),
						modifier = Modifier.size(iconSize.scaledWidth()),
					)
				},
				trailing = {
					Icon(
						Icons.AutoMirrored.Outlined.ArrowRight,
						stringResource(R.string.go),
						modifier = Modifier.size(iconSize),
					)
				},
				title = {
					Text(
						stringResource(R.string.join_discord),
						style = MaterialTheme.typography.bodyLarge.copy(MaterialTheme.colorScheme.onSurface),
					)
				},
				onClick = { context.openWebUrl(context.getString(R.string.discord_url)) },
			),
		),
	)
}
