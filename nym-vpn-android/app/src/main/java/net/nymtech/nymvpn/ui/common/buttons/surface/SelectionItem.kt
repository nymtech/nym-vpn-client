package net.nymtech.nymvpn.ui.common.buttons.surface

import androidx.compose.runtime.Composable
import androidx.compose.ui.graphics.vector.ImageVector

data class SelectionItem(
	val leadingIcon: ImageVector? = null,
	val trailing: (@Composable () -> Unit)? = null,
	val title: (@Composable () -> Unit),
	val description: (@Composable () -> Unit)? = null,
	val onClick: () -> Unit = {},
	val height: Int = 64,
)
