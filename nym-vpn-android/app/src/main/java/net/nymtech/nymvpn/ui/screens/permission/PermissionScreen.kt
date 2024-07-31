package net.nymtech.nymvpn.ui.screens.permission

import android.Manifest
import android.app.Activity.RESULT_OK
import android.net.VpnService
import android.os.Build
import androidx.activity.compose.rememberLauncherForActivityResult
import androidx.activity.result.contract.ActivityResultContracts
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Notifications
import androidx.compose.material.icons.outlined.VpnKey
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import androidx.navigation.NavController
import com.google.accompanist.permissions.ExperimentalPermissionsApi
import com.google.accompanist.permissions.isGranted
import com.google.accompanist.permissions.rememberPermissionState
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.NavItem
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
fun PermissionScreen(navController: NavController, permission: NavItem.Permission.Path) {
	val context = LocalContext.current

	val notificationPermissionState =
		if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
			rememberPermissionState(Manifest.permission.POST_NOTIFICATIONS)
		} else {
			null
		}

	val vpnActivityResultState =
		rememberLauncherForActivityResult(
			ActivityResultContracts.StartActivityForResult(),
			onResult = {
				val accepted = (it.resultCode == RESULT_OK)
				if (!accepted) {
					navController.navigate("${NavItem.Permission.route}/${NavItem.Permission.Path.VPN}")
				} else {
					navController.navigateAndForget(NavItem.Main.route)
				}
			},
		)

	LaunchedEffect(notificationPermissionState?.status) {
		if (notificationPermissionState?.status?.isGranted == true &&
			permission == NavItem.Permission.Path.NOTIFICATION
		) {
			navController.navigateAndForget(NavItem.Main.route)
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
				NavItem.Permission.Path.NOTIFICATION -> {
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
				NavItem.Permission.Path.VPN -> {
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
				}
			}
		}
		when (permission) {
			NavItem.Permission.Path.NOTIFICATION -> {
				Column(verticalArrangement = Arrangement.Bottom) {
					MainStyledButton(
						onClick = {
							notificationPermissionState?.launchPermissionRequest()
						},
						content = { Text(stringResource(R.string.allow_permissions), style = CustomTypography.labelHuge) },
					)
				}
			}
			NavItem.Permission.Path.VPN -> {
				Column(verticalArrangement = Arrangement.spacedBy(16.dp, Alignment.Bottom)) {
					MainStyledButton(
						onClick = {
							val intent = VpnService.prepare(context)
							if (intent != null) {
								vpnActivityResultState.launch(
									intent,
								)
							}
						},
						content = { Text(stringResource(R.string.allow_permissions), style = CustomTypography.labelHuge) },
					)
					MainStyledButton(
						onClick = {
							context.launchVpnSettings()
						},
						content = { Text(stringResource(R.string.view_system_settings), style = CustomTypography.labelHuge) },
					)
				}
			}
		}
	}
}
