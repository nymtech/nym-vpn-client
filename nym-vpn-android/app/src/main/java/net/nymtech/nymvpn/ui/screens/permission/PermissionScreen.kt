package net.nymtech.nymvpn.ui.screens.permission

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.foundation.text.ClickableText
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.VpnKey
import androidx.compose.material.icons.outlined.VpnLock
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.withStyle
import androidx.compose.ui.unit.dp
import androidx.navigation.NavController
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.Destination
import net.nymtech.nymvpn.ui.common.buttons.MainStyledButton
import net.nymtech.nymvpn.ui.common.buttons.surface.SelectionItem
import net.nymtech.nymvpn.ui.common.labels.PermissionLabel
import net.nymtech.nymvpn.ui.theme.CustomTypography
import net.nymtech.nymvpn.util.extensions.launchVpnSettings
import net.nymtech.nymvpn.util.extensions.navigateAndForget
import net.nymtech.nymvpn.util.extensions.scaledHeight
import net.nymtech.nymvpn.util.extensions.scaledWidth

@Composable
fun PermissionScreen(navController: NavController, permission: Permission) {
	val context = LocalContext.current

	Column(
		modifier = Modifier
			.fillMaxSize()
			.padding(horizontal = 16.dp.scaledWidth())
			.padding(vertical = 24.dp),
		horizontalAlignment = Alignment.CenterHorizontally,
		verticalArrangement = Arrangement.SpaceBetween,
	) {
		Column(verticalArrangement = Arrangement.spacedBy(32.dp.scaledHeight())) {
			Row(horizontalArrangement = Arrangement.spacedBy(16.dp.scaledWidth())) {
				Box(
					modifier = Modifier
						.width(2.dp)
						.height(60.dp)
						.background(color = MaterialTheme.colorScheme.primary, shape = RoundedCornerShape(size = 4.dp)),
				)
				Text(
					stringResource(id = R.string.permission_message),
					style = MaterialTheme.typography.bodyMedium,
					color = MaterialTheme.colorScheme.onSurfaceVariant,
				)
			}

			when (permission) {
				Permission.VPN -> {
					PermissionLabel(
						SelectionItem(
							leadingIcon = Icons.Outlined.VpnKey,
							title = { Text(stringResource(id = R.string.vpn_connection), style = MaterialTheme.typography.bodyLarge) },
							description = {
								Text(
									stringResource(id = R.string.vpn_permission_message),
									style = MaterialTheme.typography.bodyMedium,
									color = MaterialTheme.colorScheme.outline,
								)
							},
							trailing = null,
						),
					)
					val alwaysOnDescription = buildAnnotatedString {
						append(stringResource(R.string.always_on_message))
						append(" ")
						pushStringAnnotation(tag = "settings", annotation = stringResource(R.string.always_on_disbled))
						withStyle(style = SpanStyle(color = MaterialTheme.colorScheme.primary)) {
							append(stringResource(id = R.string.vpn_settings))
						}
						pop()
						append(" ")
						append(stringResource(R.string.try_again))
						append(".")
					}
					PermissionLabel(
						SelectionItem(
							leadingIcon = Icons.Outlined.VpnLock,
							title = { Text(stringResource(id = R.string.always_on_disbled), style = MaterialTheme.typography.bodyLarge) },
							description = {
								ClickableText(
									text = alwaysOnDescription,
									style = MaterialTheme.typography.bodyMedium.copy(color = MaterialTheme.colorScheme.outline),
								) {
									alwaysOnDescription.getStringAnnotations(tag = "settings", it, it).firstOrNull()?.let {
										context.launchVpnSettings()
									}
								}
							},
						),
					)
				}
			}
		}
		when (permission) {
			Permission.VPN -> {
				Column(verticalArrangement = Arrangement.spacedBy(16.dp, Alignment.Bottom)) {
					MainStyledButton(
						onClick = {
							navController.navigateAndForget(Destination.Main.createRoute(true))
						},
						content = { Text(stringResource(R.string.try_reconnecting), style = CustomTypography.labelHuge) },
					)
				}
			}
		}
	}
}
