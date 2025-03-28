package net.nymtech.nymvpn.ui.common.navigation

import androidx.compose.runtime.Composable

data class NavBarState(
	val show: Boolean = false,
	val title: @Composable () -> Unit = {},
	val leading: @Composable () -> Unit = {},
	val trailing: @Composable () -> Unit = {},
)
