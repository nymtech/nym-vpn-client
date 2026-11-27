package net.nymtech.nymvpn.ui.screens.settings.components

import android.content.res.Configuration
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Public
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.res.vectorResource
import androidx.compose.ui.tooling.preview.Preview
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.common.buttons.surface.SelectionItem
import net.nymtech.nymvpn.ui.screens.settings.SettingsActions
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme

@Composable
fun routingEntries(actions: SettingsActions): List<SettingsEntry> = buildList {
	add(
		SettingsRow(
			section = SettingsSection.ROUTING,
			title = stringResource(R.string.settings_split_tunneling_title),
			item = SelectionItem(
				leading = {
					SettingsIcon(
						ImageVector.vectorResource(R.drawable.ic_split_tunneling),
						stringResource(R.string.settings_split_tunneling_title),
					)
				},
				trailing = {
					SettingsArrowIcon()
				},
				title = {
					SettingsTitle(stringResource(R.string.settings_split_tunneling_title))
				},
				onClick = actions.onSplitTunnelingClick,
			),
		),
	)

	val geoExclusionDescription = stringResource(R.string.settings_geo_exclusion_description)
	add(
		SettingsRow(
			section = SettingsSection.ROUTING,
			title = stringResource(R.string.settings_geo_exclusion_title),
			description = geoExclusionDescription,
			item = SelectionItem(
				leading = {
					SettingsIcon(
						Icons.Outlined.Public,
						stringResource(R.string.settings_geo_exclusion_title),
					)
				},
				trailing = {
					SettingsArrowIcon()
				},
				title = {
					SettingsTitle(stringResource(R.string.settings_geo_exclusion_title))
				},
				description = {
					Text(
						geoExclusionDescription,
						style = MaterialTheme.typography.bodySmall,
						color = MaterialTheme.colorScheme.onBackground,
					)
				},
				onClick = actions.onGeoExclusionClick,
			),
		),
	)
}

@Composable
@Preview(uiMode = Configuration.UI_MODE_NIGHT_YES)
internal fun PreviewRoutingEntries() {
	NymVPNTheme(Theme.default()) {
		SettingsEntryList(entries = routingEntries(SettingsActions()))
	}
}
