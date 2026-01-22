package net.nymtech.nymvpn.ui.screens.account.create.components

import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.util.extensions.scaledHeight

@Composable
fun CreateAccountBlock(title: String, description: String? = null, button: @Composable () -> Unit) {
	Column(
		modifier = Modifier
			.fillMaxWidth(),
	) {
		Text(
			text = title,
			style = MaterialTheme.typography.titleMedium,
			color = MaterialTheme.colorScheme.onBackground,
			fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
		)

		description?.let {
			Spacer(modifier = Modifier.height(8.dp.scaledHeight()))
			Text(
				text = it,
				style = MaterialTheme.typography.bodyMedium,
				color = MaterialTheme.colorScheme.outline,
				fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
			)
		}

		Spacer(modifier = Modifier.height(24.dp.scaledHeight()))
		button()
	}
}
