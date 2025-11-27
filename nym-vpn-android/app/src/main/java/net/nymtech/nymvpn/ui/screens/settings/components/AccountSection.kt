package net.nymtech.nymvpn.ui.screens.settings.components

import android.content.res.Configuration
import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.outlined.ArrowRight
import androidx.compose.material.icons.outlined.AccountCircle
import androidx.compose.material.icons.outlined.Key
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.common.buttons.surface.SelectionItem
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.ui.theme.iconSize
import net.nymtech.nymvpn.util.extensions.scaledWidth

@Composable
fun AccountSection(isMnemonicStored: Boolean, onAccountClick: () -> Unit, onPassphraseClick: () -> Unit) {
	if (isMnemonicStored) {
		SettingsGroup(
			items = listOf(
				SelectionItem(
					leading = {
						Icon(
							Icons.Outlined.AccountCircle,
							stringResource(R.string.account),
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
							stringResource(R.string.account),
							style = MaterialTheme.typography.bodyLarge.copy(MaterialTheme.colorScheme.onSurface),
						)
					},
					onClick = onAccountClick,
				),
				SelectionItem(
					leading = {
						Icon(
							Icons.Outlined.Key,
							stringResource(R.string.settings_passphrase_title),
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
							stringResource(R.string.settings_passphrase_title),
							style = MaterialTheme.typography.bodyLarge.copy(MaterialTheme.colorScheme.onSurface),
						)
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
		AccountSection(true, {}, {})
	}
}
