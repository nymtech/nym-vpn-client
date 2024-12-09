package net.nymtech.nymvpn.ui.common.navigation

import androidx.compose.material3.CenterAlignedTopAppBar
import androidx.compose.material3.ExperimentalMaterial3Api
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.TopAppBarDefaults
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalSoftwareKeyboardController
import androidx.navigation.NavController
import androidx.navigation.compose.currentBackStackEntryAsState

@OptIn(ExperimentalMaterial3Api::class)
@Composable
fun NavBar(navBarState: NavBarState, navController: NavController, modifier: Modifier = Modifier) {
	val navBackStackEntry by navController.currentBackStackEntryAsState()

	val keyboardController = LocalSoftwareKeyboardController.current

	LaunchedEffect(navBackStackEntry) {
		keyboardController?.hide()
	}

	if (navBarState.show) {
		CenterAlignedTopAppBar(
			modifier = modifier,
			title = {
				navBarState.title()
			},
			actions = {
				navBarState.trailing()
			},
			navigationIcon = {
				navBarState.leading()
			},
			colors = TopAppBarDefaults.topAppBarColors(containerColor = MaterialTheme.colorScheme.background),
		)
	}
}
