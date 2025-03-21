package net.nymtech.nymvpn.ui.screens.permission.components

import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.VpnKey
import androidx.compose.material.icons.outlined.VpnLock
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.LinkAnnotation
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.TextLinkStyles
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.withLink
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.common.buttons.surface.SelectionItem
import net.nymtech.nymvpn.ui.common.labels.PermissionLabel
import net.nymtech.nymvpn.ui.theme.iconSize
import net.nymtech.nymvpn.util.extensions.launchVpnSettings
import net.nymtech.nymvpn.util.extensions.scaledHeight
import net.nymtech.nymvpn.util.extensions.scaledWidth

@Composable
fun VpnPermissionDetails() {
	val context = LocalContext.current

	PermissionLabel(
		SelectionItem(
			leading = {
				Icon(
					Icons.Outlined.VpnKey,
					stringResource(R.string.vpn_connection),
					modifier = Modifier.size(iconSize.scaledWidth()),
				)
			},
			title = { Text(stringResource(R.string.vpn_connection), style = MaterialTheme.typography.bodyLarge) },
			description = {
				Text(
					stringResource(R.string.vpn_permission_message),
					style = MaterialTheme.typography.bodyMedium,
					color = MaterialTheme.colorScheme.outline,
				)
			},
			trailing = null,
		),
	)
	Text(
		text = buildAnnotatedString {
			append(stringResource(R.string.always_on_message))
			append(" ")
			withLink(
				LinkAnnotation.Clickable(
					tag = stringResource(R.string.settings),
					styles = TextLinkStyles(SpanStyle(color = MaterialTheme.colorScheme.primary)),
				) {
					context.launchVpnSettings()
				},
			) {
				append(stringResource(R.string.vpn_settings))
			}
			append(" ")
			append(stringResource(R.string.try_again))
			append(".")
		},
		style = MaterialTheme.typography.bodyMedium.copy(color = MaterialTheme.colorScheme.outline),
		modifier = Modifier.padding(24.dp.scaledHeight()),
	)
	PermissionLabel(
		SelectionItem(
			leading = {
				Icon(
					Icons.Outlined.VpnLock,
					stringResource(R.string.vpn_settings),
					modifier = Modifier.size(iconSize.scaledWidth()),
				)
			},
			title = { Text(stringResource(R.string.always_on_disbled), style = MaterialTheme.typography.bodyLarge) },
			description = {},
			trailing = null,
		),
	)
}
