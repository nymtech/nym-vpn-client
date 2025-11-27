package net.nymtech.nymvpn.ui.screens.settings.components

import android.content.res.Configuration
import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.outlined.ArrowRight
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

@Composable
fun SystemStatusSection(onSystemStatusClick: () -> Unit) {
	SettingsGroup(
		items = listOf(
			SelectionItem(
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
						stringResource(R.string.settings_system_status_title),
						style = MaterialTheme.typography.bodyLarge.copy(MaterialTheme.colorScheme.onSurface),
					)
				},
				onClick = onSystemStatusClick,
			),
		),
	)
}

@Composable
@Preview(uiMode = Configuration.UI_MODE_NIGHT_YES)
internal fun PreviewSystemStatusSection() {
	NymVPNTheme(Theme.default()) {
		SystemStatusSection {}
	}
}
