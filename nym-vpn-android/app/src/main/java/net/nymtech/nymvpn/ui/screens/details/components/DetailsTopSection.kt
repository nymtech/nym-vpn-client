package net.nymtech.nymvpn.ui.screens.details.components

import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.width
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp

@Composable
internal fun DetailsTopSection(name: String, location: String, countryCode: String?, description: String?) {
	Text(
		text = name,
		style = MaterialTheme.typography.titleLarge,
		color = MaterialTheme.colorScheme.onPrimaryContainer,
	)
	Row(
		modifier = Modifier
			.padding(top = 16.dp),
		verticalAlignment = Alignment.CenterVertically,
	) {
		CountryFlag(countryCode, 16.dp)
		Spacer(modifier = Modifier.width(8.dp))
		Text(
			text = location,
			style = MaterialTheme.typography.titleMedium,
			color = MaterialTheme.colorScheme.onPrimaryContainer,
		)
	}
	description?.let {
		Text(
			modifier = Modifier.padding(top = 16.dp),
			text = it,
			style = MaterialTheme.typography.bodyMedium,
			color = MaterialTheme.colorScheme.onBackground,
		)
	}
}
