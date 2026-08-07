package net.nymtech.nymvpn.ui.screens.settings.components

import android.content.res.Configuration
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.BuildConfig
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme

@Composable
fun AppVersionSection(appVersion: String, onAppVersionClick: () -> Unit) {
	Column(
		modifier = Modifier
			.fillMaxWidth()
			.padding(bottom = 24.dp)
			.clickable {
				if (BuildConfig.DEBUG || BuildConfig.IS_PRERELEASE) {
					onAppVersionClick.invoke()
				}
			},
	) {
		Text(
			stringResource(R.string.settings_app_version_line, appVersion),
			style = MaterialTheme.typography.bodyMedium,
			color = MaterialTheme.colorScheme.outline,
		)
	}
}

@Composable
@Preview(uiMode = Configuration.UI_MODE_NIGHT_YES)
internal fun PreviewAppVersion() {
	NymVPNTheme(Theme.default()) {
		AppVersionSection("2.2.2") {}
	}
}
