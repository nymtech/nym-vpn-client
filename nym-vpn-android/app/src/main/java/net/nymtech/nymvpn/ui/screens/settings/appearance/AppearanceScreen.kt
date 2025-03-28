package net.nymtech.nymvpn.ui.screens.settings.appearance

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.ui.common.navigation.LocalNavController
import net.nymtech.nymvpn.util.extensions.scaledHeight
import net.nymtech.nymvpn.util.extensions.scaledWidth
import net.nymtech.nymvpn.ui.screens.settings.appearance.components.AppearanceOptions

@Composable
fun AppearanceScreen() {
	val navController = LocalNavController.current

	Column(
		horizontalAlignment = Alignment.Start,
		verticalArrangement = Arrangement.spacedBy(24.dp.scaledHeight(), Alignment.Top),
		modifier = Modifier
			.fillMaxSize()
			.padding(top = 24.dp.scaledHeight())
			.padding(horizontal = 24.dp.scaledWidth()),
	) {
		AppearanceOptions(navController = navController)
	}
}
