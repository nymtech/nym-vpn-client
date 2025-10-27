package net.nymtech.nymvpn.ui.screens.account.passphrase

import android.content.res.Configuration
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.AppUiState
import net.nymtech.nymvpn.ui.screens.account.passphrase.components.PassphraseCard
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.ui.theme.Typography
import net.nymtech.nymvpn.util.extensions.scaledHeight
import net.nymtech.nymvpn.util.extensions.scaledWidth

@Composable
fun PassphraseScreen(appUiState: AppUiState, viewModel: PassphraseViewModel = hiltViewModel()) {
	PassphraseScreen(
		passphrase = "",
		onShowClick = {},
	)
}

@Composable
fun PassphraseScreen(passphrase: String, onShowClick: () -> Unit) {
	Column(
		horizontalAlignment = Alignment.Start,
		verticalArrangement = Arrangement.spacedBy(24.dp.scaledHeight(), Alignment.Top),
		modifier = Modifier
			.fillMaxSize()
			.padding(top = 24.dp.scaledHeight())
			.padding(horizontal = 24.dp.scaledWidth()),
	) {
		Text(
			text = stringResource(R.string.passphrase_title),
			style = Typography.titleMedium,
			color = MaterialTheme.colorScheme.onBackground,
			fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
			modifier = Modifier
				.fillMaxWidth()
				.padding(top = 24.dp),
		)
		Text(
			text = stringResource(R.string.passphrase_description),
			style = Typography.bodyMedium,
			color = MaterialTheme.colorScheme.outline,
			fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
			modifier = Modifier
				.fillMaxWidth()
				.padding(top = 16.dp),
		)
		PassphraseCard()
	}
}

@Composable
@Preview(uiMode = Configuration.UI_MODE_NIGHT_YES)
internal fun PreviewPassphraseScreen() {
	NymVPNTheme(Theme.default()) {
		PassphraseScreen("", onShowClick = {})
	}
}
