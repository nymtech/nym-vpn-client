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
fun accountEntries(isMnemonicStored: Boolean, subscription: SubscriptionUiState?, onAccountClick: () -> Unit): List<SettingsEntry> {
	// One row opens the account page, which holds the passphrase and redeem rows.
	return buildList {
		if (isMnemonicStored) {
			add(
				R.string.settings_section_account,
				stringResource(R.string.settings_your_account_title),
				SelectionItem(
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
			)
		}
	}
}

/** The Passphrase and Redeem a code rows shown on the account page. */
@Composable
fun accountPageItems(onPassphraseClick: () -> Unit, onRedeemCodeClick: () -> Unit): List<SelectionItem> = listOf(
	SelectionItem(
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
	SelectionItem(
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
)

@Composable
@Preview(uiMode = Configuration.UI_MODE_NIGHT_YES)
internal fun PreviewAccountSection() {
	NymVPNTheme(Theme.default()) {
		SettingsGroup(
			items = accountEntries(isMnemonicStored = true, subscription = null, onAccountClick = {}).map { it.item },
		)
	}
}
