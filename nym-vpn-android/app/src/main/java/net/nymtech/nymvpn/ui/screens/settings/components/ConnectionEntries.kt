package net.nymtech.nymvpn.ui.screens.settings.components

import android.content.res.Configuration
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Dns
import androidx.compose.material.icons.outlined.Lan
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.res.vectorResource
import androidx.compose.ui.tooling.preview.Preview
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.common.buttons.ScaledSwitch
import net.nymtech.nymvpn.ui.common.buttons.surface.SelectionItem
import net.nymtech.nymvpn.ui.screens.settings.SettingsActions
import net.nymtech.nymvpn.ui.screens.settings.SettingsValues
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme

@Composable
fun connectionEntries(values: SettingsValues, actions: SettingsActions): List<SettingsEntry> = buildList {
	val bypassLanDescription = stringResource(R.string.settings_bypass_lan_description)
	add(
		SettingsRow(
			section = SettingsSection.CONNECTION,
			title = stringResource(R.string.settings_bypass_lan_title),
			description = bypassLanDescription,
			item = SelectionItem(
				leading = {
					SettingsIcon(
						Icons.Outlined.Lan,
						stringResource(R.string.settings_bypass_lan_title),
					)
				},
				trailing = {
					ScaledSwitch(
						checked = values.bypassLanEnabled,
						onClick = actions.onBypassLanEnable,
					)
				},
				title = {
					SettingsTitle(stringResource(R.string.settings_bypass_lan_title))
				},
				description = {
					Text(
						bypassLanDescription,
						style = MaterialTheme.typography.bodySmall,
						color = MaterialTheme.colorScheme.onBackground,
					)
				},
			),
		),
	)

	add(
		SettingsRow(
			section = SettingsSection.CONNECTION,
			title = stringResource(R.string.settings_censorship_title),
			item = SelectionItem(
				leading = {
					SettingsIcon(
						ImageVector.vectorResource(R.drawable.ic_censorship),
						stringResource(R.string.settings_censorship_title),
					)
				},
				trailing = {
					SettingsArrowIcon()
				},
				title = {
					SettingsTitle(stringResource(R.string.settings_censorship_title))
				},
				onClick = actions.onCensorshipClick,
			),
		),
	)

	val profilesDescription = stringResource(R.string.settings_profiles_description)
	add(
		SettingsRow(
			section = SettingsSection.CONNECTION,
			title = stringResource(R.string.settings_profiles_title),
			description = profilesDescription,
			item = SelectionItem(
				leading = {
					SettingsIcon(
						ImageVector.vectorResource(R.drawable.ic_profiles),
						stringResource(R.string.settings_profiles_title),
					)
				},
				trailing = {
					SettingsArrowIcon()
				},
				description = {
					Text(
						profilesDescription,
						style = MaterialTheme.typography.bodySmall,
						color = MaterialTheme.colorScheme.onBackground,
					)
				},
				title = {
					SettingsTitle(stringResource(R.string.settings_profiles_title))
				},
				onClick = actions.onProfilesClick,
			),
		),
	)

	add(
		SettingsRow(
			section = SettingsSection.CONNECTION,
			title = stringResource(R.string.settings_dns_title),
			item = SelectionItem(
				leading = {
					SettingsIcon(
						Icons.Outlined.Dns,
						stringResource(R.string.settings_dns_title),
					)
				},
				trailing = {
					SettingsArrowIcon()
				},
				title = {
					SettingsTitle(stringResource(R.string.settings_dns_title))
				},
				onClick = actions.onDnsClick,
			),
		),
	)
}

@Composable
@Preview(uiMode = Configuration.UI_MODE_NIGHT_YES)
internal fun PreviewConnectionEntries() {
	NymVPNTheme(Theme.default()) {
		SettingsEntryList(entries = connectionEntries(SettingsValues(), SettingsActions()))
	}
}
