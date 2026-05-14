package net.nymtech.nymvpn.ui.screens.settings.components

import android.content.res.Configuration
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.outlined.Logout
import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.common.buttons.surface.SelectionItem
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme

@Composable
fun LogoutSection(isMnemonicStored: Boolean, onLogoutClick: () -> Unit) {
	if (isMnemonicStored) {
		SettingsGroup(
			items = listOf(
				SelectionItem(
					leading = {
						SettingsIcon(
							Icons.AutoMirrored.Outlined.Logout,
							stringResource(R.string.log_out),
						)
					},
					title = {
						SettingsTitle(stringResource(R.string.log_out))
					},
					onClick = onLogoutClick,
				),
			),
		)
	}
}

@Composable
@Preview(uiMode = Configuration.UI_MODE_NIGHT_YES)
internal fun PreviewLogoutSection() {
	NymVPNTheme(Theme.default()) {
		LogoutSection(true) {}
	}
}
