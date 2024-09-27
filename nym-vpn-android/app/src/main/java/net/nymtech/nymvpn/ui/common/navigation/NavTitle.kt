package net.nymtech.nymvpn.ui.common.navigation

import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable

@Composable
fun NavTitle(text: String) {
	Text(
		text,
		style = MaterialTheme.typography.titleLarge,
	)
}
