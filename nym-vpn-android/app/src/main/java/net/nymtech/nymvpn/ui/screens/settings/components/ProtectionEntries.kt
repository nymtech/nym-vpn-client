package net.nymtech.nymvpn.ui.screens.settings.components

import android.content.res.Configuration
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Block
import androidx.compose.material.icons.outlined.Power
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
import net.nymtech.nymvpn.ui.common.functions.boldMarkupText
import net.nymtech.nymvpn.ui.screens.settings.SettingsActions
import net.nymtech.nymvpn.ui.screens.settings.SettingsValues
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme

@Composable
fun protectionEntries(values: SettingsValues, actions: SettingsActions): List<SettingsEntry> = buildList {
	val adBlockingDescription = stringResource(
		if (values.adBlockingEnabled) R.string.settings_ad_blocking_description_on else R.string.settings_ad_blocking_description_off,
	)
	add(
		SettingsRow(
			section = SettingsSection.PROTECTION,
			title = stringResource(R.string.settings_ad_blocking_title),
			description = adBlockingDescription,
			item = SelectionItem(
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
				description = {
					Text(
						adBlockingDescription,
						style = MaterialTheme.typography.bodySmall,
						color = MaterialTheme.colorScheme.onBackground,
					)
				},
			),
		),
	)

	val killSwitchDescription = boldMarkupText(stringResource(R.string.settings_kill_switch_description))
	add(
		SettingsRow(
			section = SettingsSection.PROTECTION,
			title = stringResource(R.string.settings_kill_switch_title),
			description = killSwitchDescription.text,
			item = SelectionItem(
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
						killSwitchDescription,
						style = MaterialTheme.typography.bodySmall,
						color = MaterialTheme.colorScheme.onBackground,
					)
				},
			),
		),
	)

	val mixnetTuningDescription = stringResource(R.string.settings_mixnet_tuning_description)
	add(
		SettingsRow(
			section = SettingsSection.PROTECTION,
			title = stringResource(R.string.settings_mixnet_tuning_title),
			description = mixnetTuningDescription,
			item = SelectionItem(
				leading = {
					SettingsIcon(
						ImageVector.vectorResource(R.drawable.ic_mixnet_tuning),
						stringResource(R.string.settings_mixnet_tuning_title),
					)
				},
				trailing = {
					SettingsArrowIcon()
				},
				description = {
					Text(
						mixnetTuningDescription,
						style = MaterialTheme.typography.bodySmall,
						color = MaterialTheme.colorScheme.onBackground,
					)
				},
				title = {
					SettingsTitle(stringResource(R.string.settings_mixnet_tuning_title))
				},
				onClick = actions.onMixnetTuningClick,
			),
		),
	)
}

@Composable
@Preview(uiMode = Configuration.UI_MODE_NIGHT_YES)
internal fun PreviewProtectionEntries() {
	NymVPNTheme(Theme.default()) {
		SettingsEntryList(entries = protectionEntries(SettingsValues(), SettingsActions()))
	}
}
