package net.nymtech.nymvpn.ui.screens.settings.components

import android.content.res.Configuration
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.AccountCircle
import androidx.compose.material.icons.outlined.Redeem
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.res.vectorResource
import androidx.compose.ui.tooling.preview.Preview
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.common.buttons.surface.SelectionItem
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme

@Composable
fun accountEntries(
	isMnemonicStored: Boolean,
	subscription: SubscriptionUiState?,
	onGetStartedClick: () -> Unit,
	onAccountClick: () -> Unit,
	onPassphraseClick: () -> Unit,
	onRedeemCodeClick: () -> Unit,
): List<SettingsEntry> = buildList {
	if (isMnemonicStored) {
		add(
			SettingsRow(
				section = SettingsSection.ACCOUNT,
				title = stringResource(R.string.settings_your_account_title),
				item = SelectionItem(
					leading = {
						SettingsIcon(
							Icons.Outlined.AccountCircle,
							stringResource(R.string.settings_your_account_title),
						)
					},
					trailing = {
						SettingsArrowIcon()
					},
					title = {
						SettingsTitle(stringResource(R.string.settings_your_account_title))
					},
					description = {
						SubscriptionStatusText(subscription = subscription)
						if (subscription != null && subscription.isRecurring && subscription.expiryState != ExpiryState.EXPIRED) {
							Text(
								text = stringResource(R.string.account_info_auto_renew_text),
								style = MaterialTheme.typography.bodySmall,
								color = MaterialTheme.colorScheme.onBackground,
							)
						}
					},
					onClick = onAccountClick,
				),
			),
		)
		add(
			SettingsRow(
				section = SettingsSection.ACCOUNT,
				title = stringResource(R.string.settings_passphrase_title),
				item = SelectionItem(
					leading = {
						SettingsIcon(
							ImageVector.vectorResource(R.drawable.ic_passphrase),
							stringResource(R.string.settings_passphrase_title),
						)
					},
					trailing = {
						SettingsArrowIcon()
					},
					title = {
						SettingsTitle(stringResource(R.string.settings_passphrase_title))
					},
					onClick = onPassphraseClick,
				),
			),
		)
	} else {
		add(
			SettingsBlock(
				section = SettingsSection.ACCOUNT,
				title = stringResource(R.string.get_started),
				content = { GetStartedButton(onClick = onGetStartedClick) },
			),
		)
	}
	// Redeeming without an account creates one, so it shows either way.
	add(
		SettingsRow(
			section = SettingsSection.ACCOUNT,
			title = stringResource(R.string.settings_redeem_code_title),
			item = SelectionItem(
				leading = {
					SettingsIcon(
						Icons.Outlined.Redeem,
						stringResource(R.string.settings_redeem_code_title),
					)
				},
				trailing = {
					SettingsArrowIcon()
				},
				title = {
					SettingsTitle(stringResource(R.string.settings_redeem_code_title))
				},
				onClick = onRedeemCodeClick,
			),
		),
	)
}

@Composable
@Preview(uiMode = Configuration.UI_MODE_NIGHT_YES)
internal fun PreviewAccountEntries() {
	NymVPNTheme(Theme.default()) {
		SettingsEntryList(
			entries = accountEntries(
				isMnemonicStored = true,
				subscription = null,
				onGetStartedClick = {},
				onAccountClick = {},
				onPassphraseClick = {},
				onRedeemCodeClick = {},
			),
		)
	}
}
