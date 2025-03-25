package net.nymtech.nymvpn.ui.common.navigation

import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import net.nymtech.nymvpn.R

@Composable
fun NavTitle(text: String) {
	Text(
		text.uppercase(),
		style = MaterialTheme.typography.titleLarge,
		fontFamily = FontFamily(Font(R.font.lab_grotesque_mono)),
	)
}
