package net.nymtech.nymvpn.ui.common.navigation

import androidx.compose.foundation.layout.size
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.ImageVector
import net.nymtech.nymvpn.ui.theme.iconSize

@Composable
fun NavIcon(icon: ImageVector, description: String, onClick: () -> Unit) {
	IconButton(
		onClick = {
			onClick()
		},
	) {
		Icon(
			imageVector = icon,
			contentDescription = description,
			tint = MaterialTheme.colorScheme.onSurface,
			modifier =
			Modifier.size(
				iconSize,
			),
		)
	}
}
