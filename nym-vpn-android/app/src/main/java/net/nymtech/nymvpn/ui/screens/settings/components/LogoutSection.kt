package net.nymtech.nymvpn.ui.screens.settings.components

import android.content.res.Configuration
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.common.buttons.MainStyledButton
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.util.extensions.scaledHeight

/** Neutral grey on purpose: never two red buttons together (Delete account stays red). Used in Settings and Your account. */
@Composable
fun LogoutSection(isMnemonicStored: Boolean, onLogoutClick: () -> Unit) {
	if (isMnemonicStored) {
		MainStyledButton(
			onClick = onLogoutClick,
			content = {
				Text(
					stringResource(R.string.log_out),
					style = MaterialTheme.typography.bodyLarge,
				)
			},
			color = MaterialTheme.colorScheme.primaryContainer,
			textColor = MaterialTheme.colorScheme.onPrimaryContainer,
			modifier = Modifier
				.fillMaxWidth()
				.height(48.dp.scaledHeight()),
			shape = RoundedCornerShape(12.dp),
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
