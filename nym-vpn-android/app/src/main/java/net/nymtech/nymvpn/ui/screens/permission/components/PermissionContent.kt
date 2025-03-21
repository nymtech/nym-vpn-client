package net.nymtech.nymvpn.ui.screens.permission.components

import android.net.VpnService
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import androidx.navigation.NavController
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.Route
import net.nymtech.nymvpn.ui.common.buttons.MainStyledButton
import net.nymtech.nymvpn.ui.common.snackbar.SnackbarController
import net.nymtech.nymvpn.ui.screens.permission.Permission
import net.nymtech.nymvpn.ui.theme.CustomTypography
import net.nymtech.nymvpn.util.extensions.navigateAndForget
import net.nymtech.nymvpn.util.extensions.scaledHeight

@Composable
fun PermissionContent(permission: Permission, navController: NavController) {
	val context = LocalContext.current
	val snackbar = SnackbarController.current

	Column(verticalArrangement = Arrangement.spacedBy(32.dp.scaledHeight())) {
		PermissionHeader()
		when (permission) {
			Permission.VPN -> VpnPermissionDetails()
		}
	}
	when (permission) {
		Permission.VPN -> {
			Column(verticalArrangement = Arrangement.spacedBy(16.dp, Alignment.Bottom)) {
				MainStyledButton(
					onClick = {
						if (VpnService.prepare(context) == null) {
							navController.navigateAndForget(Route.Main(true))
						} else {
							snackbar.showMessage(context.getString(R.string.permission_required))
						}
					},
					content = { Text(stringResource(R.string.try_reconnecting), style = CustomTypography.labelHuge) },
					modifier = Modifier.fillMaxWidth().height(56.dp.scaledHeight()),
				)
			}
		}
	}
}
