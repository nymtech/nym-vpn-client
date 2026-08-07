package net.nymtech.nymvpn.ui.screens.settings.components

import android.content.res.Configuration
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Dns
import androidx.compose.material.icons.outlined.Lan
import androidx.compose.material.icons.outlined.Block
import androidx.compose.material.icons.outlined.Power
import androidx.compose.material.icons.outlined.Public
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
fun VpnSettingsSection(values: SettingsValues, actions: SettingsActions) {
	SettingsGroup(
		items = buildList {
			add(
				SelectionItem(
					leading = {
						SettingsIcon(
							Icons.Outlined.Power,
							stringResource(R.string.settings_kill_switch_title),
						)
					},
					trailing = {
						SettingsArrowIcon()
					},
					title = {
						SettingsTitle(stringResource(R.string.settings_kill_switch_title))
					},
					onClick = actions.onKillSwitchClick,
					description = {
						Text(
							stringResource(R.string.settings_kill_switch_description),
							style = MaterialTheme.typography.bodySmall,
							color = MaterialTheme.colorScheme.onBackground,
						)
					},
				),
			)

			add(
				SelectionItem(
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
							stringResource(R.string.settings_bypass_lan_description),
							style = MaterialTheme.typography.bodySmall,
							color = MaterialTheme.colorScheme.onBackground,
						)
					},
				),
			)

			add(
				SelectionItem(
					leading = {
						SettingsIcon(
							Icons.Outlined.Block,
							stringResource(R.string.settings_ad_blocking_title),
						)
					},
					trailing = {
						ScaledSwitch(
							checked = values.adBlockingEnabled,
							onClick = actions.onAdBlockingEnable,
						)
					},
					title = {
						SettingsTitle(stringResource(R.string.settings_ad_blocking_title))
					},
				),
			)

			add(
				SelectionItem(
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
							stringResource(R.string.settings_geo_exclusion_desciption),
							style = MaterialTheme.typography.bodySmall,
							color = MaterialTheme.colorScheme.onBackground,
						)
					},
					onClick = actions.onGeoExclusionClick,
				),
			)

			add(
				SelectionItem(
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
			)

			add(
				SelectionItem(
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
			)
			add(
				SelectionItem(
					leading = {
						SettingsIcon(
							ImageVector.vectorResource(R.drawable.ic_mixnet_tuning),
							stringResource(R.string.settings_split_tunneling_title),
						)
					},
					trailing = {
						SettingsArrowIcon()
					},
					description = {
						Text(
							stringResource(R.string.settings_mixnet_tuning_description),
							style = MaterialTheme.typography.bodySmall,
							color = MaterialTheme.colorScheme.onBackground,
						)
					},
					title = {
						SettingsTitle(stringResource(R.string.settings_mixnet_tuning_title))
					},
					onClick = actions.onMixnetTuningClick,
				),
			)
			add(
				SelectionItem(
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
			)
		},
	)
}

@Composable
@Preview(uiMode = Configuration.UI_MODE_NIGHT_YES)
internal fun PreviewVpnSections() {
	NymVPNTheme(Theme.default()) {
		VpnSettingsSection(
			SettingsValues(),
			SettingsActions(),
		)
	}
}
