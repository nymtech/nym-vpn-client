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
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.res.vectorResource
import androidx.compose.ui.tooling.preview.Preview
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.common.buttons.surface.SelectionItem
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.ui.theme.iconSize
import net.nymtech.nymvpn.util.extensions.scaledWidth

@Composable
fun ResetAppSection(onResetAppClick: () -> Unit) {
	SettingsGroup(
		items = listOf(
			SelectionItem(
				leading = {
					Icon(
						ImageVector.vectorResource(R.drawable.ic_reset_app),
						stringResource(R.string.settings_reset_app_title),
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
						stringResource(R.string.settings_reset_app_title),
						style = MaterialTheme.typography.bodyLarge.copy(MaterialTheme.colorScheme.onSurface),
					)
				},
				onClick = onResetAppClick,
			),
		),
	)
}

@Composable
@Preview(uiMode = Configuration.UI_MODE_NIGHT_YES)
internal fun PreviewResetAppSection() {
	NymVPNTheme(Theme.default()) {
		ResetAppSection {}
	}
}
