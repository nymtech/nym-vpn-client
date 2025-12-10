package net.nymtech.nymvpn.ui.screens.settings.components

import android.content.res.Configuration
import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.outlined.ArrowRight
import androidx.compose.material.icons.outlined.Campaign
import androidx.compose.material.icons.automirrored.outlined.CallSplit
import androidx.compose.material.icons.outlined.Dns
import androidx.compose.material.icons.outlined.Lan
import androidx.compose.material.icons.outlined.Power
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
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
import net.nymtech.nymvpn.ui.theme.iconSize
import net.nymtech.nymvpn.util.extensions.scaledWidth

@Composable
fun VpnSettingsSection(values: SettingsValues, actions: SettingsActions) {
	SettingsGroup(
		items = buildList {
			add(
				SelectionItem(
					leading = {
						Icon(
							ImageVector.vectorResource(R.drawable.auto),
							stringResource(R.string.auto_connect),
							modifier = Modifier.size(iconSize.scaledWidth()),
							tint = MaterialTheme.colorScheme.outline,
						)
					},
					trailing = {
						ScaledSwitch(
							checked = values.autoConnectEnabled,
							onClick = actions.onAutoConnectEnable,
						)
					},
					title = {
						Text(
							stringResource(R.string.auto_connect),
							style = MaterialTheme.typography.bodyLarge.copy(MaterialTheme.colorScheme.onSurface),
						)
					},
					description = {
						Text(
							stringResource(R.string.auto_connect_description),
							style = MaterialTheme.typography.bodySmall.copy(MaterialTheme.colorScheme.outline),
						)
					},
				),
			)

// 			add(
// 				SelectionItem(
// 					leading = {
// 						Icon(
// 							Icons.Outlined.AddModerator,
// 							stringResource(R.string.settings_ipv6_title),
// 							modifier = Modifier.size(iconSize.scaledWidth()),
// 							tint = MaterialTheme.colorScheme.outline,
// 						)
// 					},
// 					trailing = {
// 						ScaledSwitch(
// 							checked = values.supportIPv6Enabled,
// 							onClick = actions.onSupportIPv6Enable,
// 						)
// 					},
// 					title = {
// 						Text(
// 							stringResource(R.string.settings_ipv6_title),
// 							style = MaterialTheme.typography.bodyLarge.copy(MaterialTheme.colorScheme.onSurface),
// 						)
// 					},
// 					description = {
// 						Text(
// 							stringResource(R.string.settings_ipv6_description),
// 							style = MaterialTheme.typography.bodySmall.copy(MaterialTheme.colorScheme.outline),
// 						)
// 					},
// 				),
// 			)
			add(
				SelectionItem(
					leading = {
						Icon(
							Icons.Outlined.Lan,
							stringResource(R.string.bypass_lan),
							modifier = Modifier.size(iconSize.scaledWidth()),
							tint = MaterialTheme.colorScheme.outline,
						)
					},
					trailing = {
						ScaledSwitch(
							checked = values.bypassLanEnabled,
							onClick = actions.onBypassLanEnable,
						)
					},
					title = {
						Text(
							stringResource(R.string.bypass_lan),
							style = MaterialTheme.typography.bodyLarge.copy(MaterialTheme.colorScheme.onSurface),
						)
					},
					description = {
						Text(
							stringResource(R.string.settings_bypass_description),
							style = MaterialTheme.typography.bodySmall.copy(MaterialTheme.colorScheme.outline),
						)
					},
				),
			)

// 			add(
// 				SelectionItem(
// 					leading = {
// 						Icon(
// 							Icons.Outlined.LooksTwo,
// 							stringResource(R.string.settings_auto_select_title),
// 							modifier = Modifier.size(iconSize.scaledWidth()),
// 							tint = MaterialTheme.colorScheme.outline,
// 						)
// 					},
// 					trailing = {
// 						ScaledSwitch(
// 							checked = values.autoselectServerEnabled,
// 							onClick = actions.onAutoselectServerEnable,
// 						)
// 					},
// 					title = {
// 						Text(
// 							stringResource(R.string.settings_auto_select_title),
// 							style = MaterialTheme.typography.bodyLarge.copy(MaterialTheme.colorScheme.onSurface),
// 						)
// 					},
// 					description = {
// 						Text(
// 							stringResource(R.string.settings_auto_select_description),
// 							style = MaterialTheme.typography.bodySmall.copy(MaterialTheme.colorScheme.outline),
// 						)
// 					},
// 				),
// 			)

			add(
				SelectionItem(
					leading = {
						Icon(
							Icons.Outlined.Power,
							stringResource(R.string.kill_switch),
							modifier = Modifier.size(iconSize.scaledWidth()),
							tint = MaterialTheme.colorScheme.outline,
						)
					},
					trailing = {
						Icon(
							Icons.AutoMirrored.Outlined.ArrowRight,
							stringResource(R.string.go),
							modifier = Modifier.size(iconSize),
							tint = MaterialTheme.colorScheme.onBackground,
						)
					},
					title = {
						Text(
							stringResource(R.string.kill_switch),
							style = MaterialTheme.typography.bodyLarge.copy(MaterialTheme.colorScheme.onSurface),
						)
					},
					onClick = actions.onKillSwitchClick,
				),
			)
			add(
				SelectionItem(
					leading = {
						Icon(
							Icons.AutoMirrored.Outlined.CallSplit,
							stringResource(R.string.split_tunneling),
							modifier = Modifier.size(iconSize.scaledWidth()),
							tint = MaterialTheme.colorScheme.outline,
						)
					},
					trailing = {
						Icon(
							Icons.AutoMirrored.Outlined.ArrowRight,
							stringResource(R.string.go),
							modifier = Modifier.size(iconSize),
							tint = MaterialTheme.colorScheme.onBackground,
						)
					},
					title = {
						Text(
							stringResource(R.string.split_tunneling),
							style = MaterialTheme.typography.bodyLarge.copy(MaterialTheme.colorScheme.onSurface),
						)
					},
					onClick = actions.onSplitTunnelingClick,
				),
			)

			add(
				SelectionItem(
					leading = {
						Icon(
							Icons.Outlined.Dns,
							stringResource(R.string.settings_dns_title),
							modifier = Modifier.size(iconSize.scaledWidth()),
							tint = MaterialTheme.colorScheme.outline,
						)
					},
					trailing = {
						Icon(
							Icons.AutoMirrored.Outlined.ArrowRight,
							stringResource(R.string.go),
							modifier = Modifier.size(iconSize),
							tint = MaterialTheme.colorScheme.onBackground,
						)
					},
					title = {
						Text(
							stringResource(R.string.settings_dns_title),
							style = MaterialTheme.typography.bodyLarge.copy(MaterialTheme.colorScheme.onSurface),
						)
					},
					onClick = actions.onDnsClick,
				),
			)

			if (values.showCensorshipSection) {
				add(
					SelectionItem(
						leading = {
							Icon(
								Icons.Outlined.Campaign,
								stringResource(R.string.settings_censorship_title),
								modifier = Modifier.size(iconSize.scaledWidth()),
								tint = MaterialTheme.colorScheme.outline,
							)
						},
						trailing = {
							Icon(
								Icons.AutoMirrored.Outlined.ArrowRight,
								stringResource(R.string.go),
								modifier = Modifier.size(iconSize),
								tint = MaterialTheme.colorScheme.onBackground,
							)
						},
						title = {
							Text(
								stringResource(R.string.settings_censorship_title),
								style = MaterialTheme.typography.bodyLarge.copy(MaterialTheme.colorScheme.onSurface),
							)
						},
						onClick = actions.onCensorshipClick,
					),
				)
			}
		},
	)
}

@Composable
@Preview(uiMode = Configuration.UI_MODE_NIGHT_YES)
internal fun PreviewVpnSections() {
	NymVPNTheme(Theme.default()) {
		VpnSettingsSection(
			SettingsValues(showCensorshipSection = true),
			SettingsActions(),
		)
	}
}
