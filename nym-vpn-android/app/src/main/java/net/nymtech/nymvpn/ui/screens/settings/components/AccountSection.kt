package net.nymtech.nymvpn.ui.screens.settings.components

import android.content.res.Configuration
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.AccountCircle
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
fun AccountSection(isMnemonicStored: Boolean, subscription: SubscriptionUiState?, onAccountClick: () -> Unit, onPassphraseClick: () -> Unit) {
	if (isMnemonicStored) {
		SettingsGroup(
			items = listOf(
				SelectionItem(
					leading = {
						SettingsIcon(
							Icons.Outlined.AccountCircle,
							stringResource(R.string.account),
						)
					},
					trailing = {
						SettingsArrowIcon()
					},
					title = {
						SettingsTitle(stringResource(R.string.account))
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
			),
		)
	}
}

@Composable
@Preview(uiMode = Configuration.UI_MODE_NIGHT_YES)
internal fun PreviewAccountSection() {
	NymVPNTheme(Theme.default()) {
		AccountSection(isMnemonicStored = true, subscription = null, onAccountClick = {}, onPassphraseClick = {})
	}
}
