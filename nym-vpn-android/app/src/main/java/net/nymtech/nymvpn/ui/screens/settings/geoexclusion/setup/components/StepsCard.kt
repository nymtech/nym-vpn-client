package net.nymtech.nymvpn.ui.screens.settings.geoexclusion.setup.components

import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.wrapContentHeight
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.unit.dp

@Composable
fun StepsCard(steps: List<AnnotatedString>, modifier: Modifier = Modifier) {
	Card(
		modifier = modifier
			.fillMaxWidth()
			.wrapContentHeight(),
		shape = RoundedCornerShape(14.dp),
		colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.primaryContainer),
	) {
		steps.forEachIndexed { index, step ->
			if (index > 0) {
				HorizontalDivider(color = MaterialTheme.colorScheme.outlineVariant, thickness = 1.dp)
			}
			Row(
				modifier = Modifier
					.fillMaxWidth()
					.padding(16.dp),
				verticalAlignment = Alignment.CenterVertically,
			) {
				Card(
					modifier = Modifier.size(40.dp),
					shape = CircleShape,
					colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.primary.copy(alpha = 0.1f)),
					border = BorderStroke(1.dp, MaterialTheme.colorScheme.primary),
				) {
					Row(
						modifier = Modifier.fillMaxSize(),
						verticalAlignment = Alignment.CenterVertically,
						horizontalArrangement = Arrangement.Center,
					) {
						Text(
							text = (index + 1).toString(),
							color = MaterialTheme.colorScheme.primary,
							style = MaterialTheme.typography.labelLarge,
						)
					}
				}
				Text(
					text = step,
					modifier = Modifier
						.weight(1f)
						.padding(start = 16.dp),
					color = MaterialTheme.colorScheme.onPrimaryContainer,
					style = MaterialTheme.typography.bodyMedium,
				)
			}
		}
	}
}
