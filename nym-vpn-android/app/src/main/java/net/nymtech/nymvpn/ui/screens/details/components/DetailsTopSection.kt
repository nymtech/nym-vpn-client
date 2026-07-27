package net.nymtech.nymvpn.ui.screens.details.components

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.ui.theme.iconSize
import net.nymtech.nymvpn.util.extensions.FavoriteIcon

@Composable
internal fun DetailsTopSection(name: String, location: String, countryCode: String?, description: String?, isFavorite: Boolean, onToggleFavorite: () -> Unit) {
	Column(
		modifier = Modifier
			.fillMaxWidth()
			.background(
				color = MaterialTheme.colorScheme.primaryContainer,
				shape = RoundedCornerShape(size = 14.dp),
			),
	) {
		Row(
			modifier = Modifier
				.fillMaxWidth()
				.padding(16.dp),
			verticalAlignment = Alignment.CenterVertically,
		) {
			CountryFlag(countryCode, 26.dp)
			Spacer(modifier = Modifier.width(8.dp))
			Text(
				text = name,
				style = MaterialTheme.typography.titleMedium,
				color = MaterialTheme.colorScheme.onPrimaryContainer,
				modifier = Modifier.weight(1f),
			)
			FavoriteIcon(
				isFavorite = isFavorite,
				onToggleFavorite = onToggleFavorite,
				modifier = Modifier.size(iconSize),
			)
		}
		HorizontalDivider(
			modifier = Modifier
				.fillMaxWidth()
				.background(color = MaterialTheme.colorScheme.surfaceVariant)
				.height(1.dp),
		)
		Column(modifier = Modifier.padding(16.dp)) {
			Text(
				text = location,
				style = MaterialTheme.typography.titleSmall,
				color = MaterialTheme.colorScheme.onPrimaryContainer,
			)
			description?.let {
				Text(
					modifier = Modifier.padding(top = 12.dp),
					text = it,
					style = MaterialTheme.typography.bodyMedium,
					color = MaterialTheme.colorScheme.onBackground,
				)
			}
		}
	}
}
