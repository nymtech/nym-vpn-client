package net.nymtech.nymvpn.ui.screens.account.delete.components

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.unit.dp

@Composable
fun DeleteSection(title: String, items: List<AnnotatedString>) {
	Column(verticalArrangement = Arrangement.spacedBy(8.dp)) {
		Text(
			text = title,
			style = MaterialTheme.typography.titleMedium,
			color = MaterialTheme.colorScheme.onPrimaryContainer,
		)
		items.forEach { item ->
			Row(horizontalArrangement = Arrangement.spacedBy(8.dp)) {
				Text(
					text = "•",
					style = MaterialTheme.typography.bodyMedium,
					color = MaterialTheme.colorScheme.onBackground,
				)
				Text(
					text = item,
					style = MaterialTheme.typography.bodyMedium,
					color = MaterialTheme.colorScheme.onBackground,
				)
			}
		}
	}
}
