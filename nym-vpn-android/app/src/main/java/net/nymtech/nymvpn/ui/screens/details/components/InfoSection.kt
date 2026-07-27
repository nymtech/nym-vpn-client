package net.nymtech.nymvpn.ui.screens.details.components

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp

@Composable
fun InfoSection(titleResId: Int, items: List<Pair<String, @Composable () -> Unit>>, bottomContent: (@Composable () -> Unit)? = null, modifier: Modifier = Modifier) {
	Column(
		modifier = modifier
			.fillMaxWidth()
			.background(
				color = MaterialTheme.colorScheme.primaryContainer,
				shape = RoundedCornerShape(size = 14.dp),
			)
			.padding(16.dp),
	) {
		Text(
			text = stringResource(titleResId),
			style = MaterialTheme.typography.labelLarge,
			color = MaterialTheme.colorScheme.primary,
		)
		items.forEachIndexed { index, item ->
			Row(
				modifier = Modifier
					.fillMaxWidth()
					.padding(vertical = 10.dp),
				horizontalArrangement = Arrangement.SpaceBetween,
				verticalAlignment = Alignment.CenterVertically,
			) {
				Text(
					text = item.first,
					style = MaterialTheme.typography.bodyMedium,
					color = MaterialTheme.colorScheme.onBackground,
				)

				item.second()
			}

			if (index != items.lastIndex) {
				HorizontalDivider(
					modifier = Modifier
						.fillMaxWidth()
						.background(color = MaterialTheme.colorScheme.surfaceVariant)
						.height(1.dp),
				)
			}
		}

		bottomContent?.let {
			Box(
				modifier = Modifier.padding(top = 4.dp),
			) {
				it()
			}
		}
	}
}
