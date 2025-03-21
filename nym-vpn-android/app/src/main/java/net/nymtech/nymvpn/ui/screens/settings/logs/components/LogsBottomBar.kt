package net.nymtech.nymvpn.ui.screens.settings.logs.components

import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Delete
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

@Composable
fun LogsBottomBar(onShareClick: () -> Unit, onDeleteClick: () -> Unit) {
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
				onClick = onShareClick,
				label = { Text(stringResource(R.string.share), style = MaterialTheme.typography.labelMedium) },
				icon = { Icon(Icons.Outlined.Share, stringResource(R.string.share)) },
			),
			NavigationBarItem(
				colors = NavigationBarItemDefaults.colors().copy(
					unselectedIconColor = MaterialTheme.colorScheme.onSurface,
					unselectedTextColor = MaterialTheme.colorScheme.onSurface,
				),
				selected = false,
				onClick = onDeleteClick,
				label = { Text(stringResource(R.string.delete), style = MaterialTheme.typography.labelMedium) },
				icon = { Icon(Icons.Outlined.Delete, stringResource(R.string.delete)) },
			),
		)
	}
}
