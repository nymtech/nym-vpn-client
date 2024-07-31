package net.nymtech.nymvpn.ui.screens.permission

import android.Manifest
import android.os.Build
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.text.ClickableText
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Notifications
import androidx.compose.material.icons.outlined.VpnKey
import androidx.compose.material.icons.outlined.VpnLock
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.SpanStyle
import androidx.compose.ui.text.buildAnnotatedString
import androidx.compose.ui.text.withStyle
import androidx.compose.ui.unit.dp
import androidx.navigation.NavController
import com.google.accompanist.permissions.ExperimentalPermissionsApi
import com.google.accompanist.permissions.isGranted
import com.google.accompanist.permissions.rememberPermissionState
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

@OptIn(ExperimentalPermissionsApi::class)
@Composable
fun PermissionScreen(navController: NavController, permission: Permission) {
	val context = LocalContext.current

	val notificationPermissionState =
		if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
			rememberPermissionState(Manifest.permission.POST_NOTIFICATIONS)
		} else {
			null
		}

	LaunchedEffect(notificationPermissionState?.status) {
		if (notificationPermissionState?.status?.isGranted == true &&
			permission == Permission.NOTIFICATION
		) {
			navController.navigateAndForget(Destination.Main.route)
		}
	}

	Column(
		modifier = Modifier
			.fillMaxSize()
			.padding(horizontal = 16.dp.scaledWidth())
			.padding(vertical = 24.dp),
		horizontalAlignment = Alignment.CenterHorizontally,
		verticalArrangement = Arrangement.SpaceBetween,
	) {
		Column(verticalArrangement = Arrangement.spacedBy(32.dp.scaledHeight())) {
			Text(
				stringResource(id = R.string.permission_message),
				style = MaterialTheme.typography.bodyLarge,
			)
			when (permission) {
				Permission.NOTIFICATION -> {
					PermissionLabel(
						SelectionItem(
							leadingIcon = Icons.Outlined.Notifications,
							title = { Text(stringResource(id = R.string.notifications), style = MaterialTheme.typography.bodyLarge) },
							description = {
								Text(
									stringResource(id = R.string.notification_permission_message),
									style = MaterialTheme.typography.bodyMedium,
									color = MaterialTheme.colorScheme.outline,
								)
							},
							trailing = null,
						),
					)
				}
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
			Permission.NOTIFICATION -> {
				Column(verticalArrangement = Arrangement.Bottom) {
					MainStyledButton(
						onClick = {
							notificationPermissionState?.launchPermissionRequest()
						},
						content = { Text(stringResource(R.string.allow_permissions), style = CustomTypography.labelHuge) },
					)
				}
			}
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
