package net.nymtech.nymvpn.ui.screens.permission

import android.Manifest
import android.os.Build
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Notifications
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
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
import net.nymtech.nymvpn.util.navigateNoBack
import net.nymtech.nymvpn.util.scaledHeight
import net.nymtech.nymvpn.util.scaledWidth

@OptIn(ExperimentalPermissionsApi::class)
@Composable
fun PermissionScreen(navController: NavController) {
	val notificationPermissionState =
		if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
			rememberPermissionState(Manifest.permission.POST_NOTIFICATIONS)
		} else {
			null
		}

	LaunchedEffect(notificationPermissionState?.status) {
		if (notificationPermissionState?.status?.isGranted == true) {
			navController.navigateNoBack(NavItem.Main.route)
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
		Column(verticalArrangement = Arrangement.Bottom) {
			MainStyledButton(
				onClick = {
					notificationPermissionState?.launchPermissionRequest()
				},
				content = { Text(stringResource(R.string.allow_permissions), style = CustomTypography.labelHuge) },
			)
		}
	}
}
