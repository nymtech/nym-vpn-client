package net.nymtech.nymvpn.ui.screens.settings.components

import android.content.res.Configuration
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.common.buttons.MainStyledButton
import net.nymtech.nymvpn.ui.theme.CustomTypography
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.util.extensions.scaledHeight

@Composable
fun LoginSection(isMnemonicStored: Boolean, onLoginClick: () -> Unit) {
	if (!isMnemonicStored) {
		MainStyledButton(
			onClick = onLoginClick,
			content = {
				Text(
					stringResource(R.string.log_in),
					style = CustomTypography.buttonMain,
				)
			},
			modifier = Modifier
				.fillMaxWidth()
				.height(56.dp.scaledHeight()),
		)
	}
}

@Composable
@Preview(uiMode = Configuration.UI_MODE_NIGHT_YES)
internal fun PreviewLoginSection() {
	NymVPNTheme(Theme.default()) {
		LoginSection( false) {}
	}
}
