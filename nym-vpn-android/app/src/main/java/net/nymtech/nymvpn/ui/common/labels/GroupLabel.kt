package net.nymtech.nymvpn.ui.common.labels

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import net.nymtech.nymvpn.R

@Composable
fun GroupLabel(title: String) {
	Row(
		verticalAlignment = Alignment.CenterVertically,
		horizontalArrangement = Arrangement.Start,
	) {
		Text(
			title.uppercase(),
			style = MaterialTheme.typography.titleMedium,
			fontFamily = FontFamily(Font(R.font.lab_grotesque_mono)),
		)
	}
}
