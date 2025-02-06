package net.nymtech.nymvpn.ui.common.buttons.surface

import androidx.compose.runtime.Composable

data class SelectionItem(
	val leading: (@Composable () -> Unit)? = null,
	val trailing: (@Composable () -> Unit)? = null,
	val title: (@Composable () -> Unit),
	val description: (@Composable () -> Unit)? = null,
	val onClick: () -> Unit = {},
	val height: Int = 64,
	val selected: Boolean = false,
)
