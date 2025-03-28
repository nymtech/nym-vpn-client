package net.nymtech.nymvpn.ui.screens.permission

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.navigationBars
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.windowInsetsPadding
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.ui.common.navigation.LocalNavController
import net.nymtech.nymvpn.util.extensions.scaledWidth
import net.nymtech.nymvpn.ui.screens.permission.components.PermissionContent

@Composable
fun PermissionScreen(permission: Permission) {
	val navController = LocalNavController.current

	Column(
		modifier = Modifier
			.fillMaxSize()
			.padding(horizontal = 16.dp.scaledWidth())
			.padding(vertical = 24.dp)
			.windowInsetsPadding(WindowInsets.navigationBars),
		horizontalAlignment = Alignment.CenterHorizontally,
		verticalArrangement = Arrangement.SpaceBetween,
	) {
		PermissionContent(permission = permission, navController = navController)
	}
}
