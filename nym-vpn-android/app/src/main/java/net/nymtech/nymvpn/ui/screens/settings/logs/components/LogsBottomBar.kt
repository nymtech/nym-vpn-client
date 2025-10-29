package net.nymtech.nymvpn.ui.screens.settings.logs.components

import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Delete
import androidx.compose.material.icons.outlined.Download
import androidx.compose.material.icons.outlined.Share
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.NavigationBar
import androidx.compose.material3.NavigationBarItem
import androidx.compose.material3.NavigationBarItemDefaults
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.screens.settings.logs.LogsBottomBarEvent

@Composable
fun LogsBottomBar(onEvent: (LogsBottomBarEvent) -> Unit) {
	NavigationBar(
		containerColor = MaterialTheme.colorScheme.surface,
		tonalElevation = 0.dp,
	) {
		listOf(
			NavigationBarItem(
				colors = NavigationBarItemDefaults.colors().copy(
					unselectedIconColor = MaterialTheme.colorScheme.onSurface,
					unselectedTextColor = MaterialTheme.colorScheme.onSurface,
				),
				selected = false,
				onClick = { onEvent(LogsBottomBarEvent.Download) },
				label = { Text(stringResource(R.string.logs_download), style = MaterialTheme.typography.labelMedium) },
				icon = { Icon(Icons.Outlined.Download, stringResource(R.string.logs_download)) },
			),
			NavigationBarItem(
				colors = NavigationBarItemDefaults.colors().copy(
					unselectedIconColor = MaterialTheme.colorScheme.onSurface,
					unselectedTextColor = MaterialTheme.colorScheme.onSurface,
				),
				selected = false,
				onClick = { onEvent(LogsBottomBarEvent.Share) },
				label = { Text(stringResource(R.string.logs_share), style = MaterialTheme.typography.labelMedium) },
				icon = { Icon(Icons.Outlined.Share, stringResource(R.string.logs_share)) },
			),
			NavigationBarItem(
				colors = NavigationBarItemDefaults.colors().copy(
					unselectedIconColor = MaterialTheme.colorScheme.onSurface,
					unselectedTextColor = MaterialTheme.colorScheme.onSurface,
				),
				selected = false,
				onClick = { onEvent(LogsBottomBarEvent.Delete) },
				label = { Text(stringResource(R.string.logs_delete), style = MaterialTheme.typography.labelMedium) },
				icon = { Icon(Icons.Outlined.Delete, stringResource(R.string.logs_delete)) },
			),
		)
	}
}
