package net.nymtech.nymvpn.ui.common.navigation

import androidx.compose.foundation.layout.size
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import net.nymtech.nymvpn.ui.theme.LocalNymColors
import net.nymtech.nymvpn.ui.theme.iconSize

@Composable
fun NavIcon(icon: ImageVector, description: String, onClick: () -> Unit, tint: Color = LocalNymColors.current.navBarIconTint) {
	IconButton(
		onClick = {
			onClick()
		},
	) {
		Icon(
			imageVector = icon,
			contentDescription = description,
			tint = tint,
			modifier =
			Modifier.size(
				iconSize,
			),
		)
	}
}
