package net.nymtech.nymvpn.ui.screens.settings.support

import android.content.res.Configuration
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.screens.settings.components.SettingsTitle
import net.nymtech.nymvpn.ui.screens.settings.support.components.SupportOptions
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.util.extensions.scaledHeight
import net.nymtech.nymvpn.util.extensions.scaledWidth

@Composable
fun SupportScreen() {
	Column(
		horizontalAlignment = Alignment.Start,
		verticalArrangement = Arrangement.spacedBy(24.dp, Alignment.Top),
		modifier = Modifier
			.verticalScroll(rememberScrollState())
			.fillMaxSize()
			.padding(vertical = 24.dp.scaledHeight())
			.padding(horizontal = 24.dp.scaledWidth()),
	) {
		SettingsTitle(stringResource(R.string.support_protect_title))
		Text(
			text = stringResource(R.string.support_protect_description),
			style = MaterialTheme.typography.bodyMedium.copy(MaterialTheme.colorScheme.onBackground),
		)
		SupportOptions()
	}
}

@Composable
@Preview(uiMode = Configuration.UI_MODE_NIGHT_YES)
internal fun PreviewSupportScreen() {
	NymVPNTheme(Theme.default()) {
		SupportScreen()
	}
}
