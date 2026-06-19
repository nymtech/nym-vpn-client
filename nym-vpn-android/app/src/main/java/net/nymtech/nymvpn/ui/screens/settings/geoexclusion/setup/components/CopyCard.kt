package net.nymtech.nymvpn.ui.screens.settings.geoexclusion.setup.components

import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.wrapContentHeight
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.ContentCopy
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp

@Composable
fun CopyCard(title: String, value: String, onClick: () -> Unit, modifier: Modifier = Modifier) {
	Card(
		shape = RoundedCornerShape(14.dp),
		colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.primaryContainer),
		modifier = modifier
			.fillMaxWidth()
			.wrapContentHeight(),
	) {
		Column(modifier = Modifier.fillMaxWidth().padding(16.dp)) {
			Text(
				text = title,
				color = MaterialTheme.colorScheme.onBackground,
				style = MaterialTheme.typography.bodySmall,
			)
			Row(
				modifier = Modifier
					.fillMaxWidth()
					.padding(top = 8.dp),
				verticalAlignment = Alignment.CenterVertically,
			) {
				Text(
					text = value,
					style = MaterialTheme.typography.bodyMedium,
					color = MaterialTheme.colorScheme.onPrimaryContainer,
					modifier = Modifier.weight(1f),
				)
				Icon(
					imageVector = Icons.Outlined.ContentCopy,
					contentDescription = title,
					modifier = Modifier
						.padding(start = 8.dp)
						.size(16.dp)
						.clickable { onClick() },
					tint = MaterialTheme.colorScheme.primary,
				)
			}
		}
	}
}
