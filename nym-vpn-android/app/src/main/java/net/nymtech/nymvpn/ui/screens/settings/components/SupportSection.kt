package net.nymtech.nymvpn.ui.screens.settings.components

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
import androidx.navigation.NavController
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.Route
import net.nymtech.nymvpn.ui.common.buttons.surface.SelectionItem
import net.nymtech.nymvpn.ui.theme.iconSize
import net.nymtech.nymvpn.util.extensions.scaledWidth

@Composable
fun SupportSection(navController: NavController) {
	SettingsGroup(
		items = listOf(
			SelectionItem(
				leading = {
					Icon(
						ImageVector.vectorResource(R.drawable.support),
						stringResource(R.string.support),
						modifier = Modifier.size(iconSize.scaledWidth()),
					)
				},
				trailing = {
					Icon(
						Icons.AutoMirrored.Outlined.ArrowRight,
						stringResource(R.string.go),
						modifier = Modifier.size(iconSize),
					)
				},
				title = {
					Text(
						stringResource(R.string.support_and_feedback),
						style = MaterialTheme.typography.bodyLarge.copy(MaterialTheme.colorScheme.onSurface),
					)
				},
				onClick = { navController.navigate(Route.Support) },
			),
			SelectionItem(
				leading = {
					Icon(
						ImageVector.vectorResource(R.drawable.logs),
						stringResource(R.string.logs),
						modifier = Modifier.size(iconSize.scaledWidth()),
					)
				},
				trailing = {
					Icon(
						Icons.AutoMirrored.Outlined.ArrowRight,
						stringResource(R.string.go),
						modifier = Modifier.size(iconSize),
					)
				},
				title = {
					Text(
						stringResource(R.string.local_logs),
						style = MaterialTheme.typography.bodyLarge.copy(MaterialTheme.colorScheme.onSurface),
					)
				},
				onClick = { navController.navigate(Route.Logs) },
			),
		),
	)
}
