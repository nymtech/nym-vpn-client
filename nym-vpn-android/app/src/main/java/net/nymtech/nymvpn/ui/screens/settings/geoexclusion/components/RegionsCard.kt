package net.nymtech.nymvpn.ui.screens.settings.geoexclusion.components

import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.clickable
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.layout.wrapContentHeight
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.KeyboardArrowRight
import androidx.compose.material.icons.filled.Add
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.screens.settings.components.SettingsTitle
import net.nymtech.nymvpn.ui.theme.iconSize

@Composable
fun RegionsCard(onRegionClick: () -> Unit, onAddRegionClick: () -> Unit) {
	Card(
		shape = RoundedCornerShape(14.dp),
		colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.primaryContainer),
		modifier = Modifier
			.fillMaxWidth()
			.wrapContentHeight(),
	) {
		Column {
			Row(
				modifier = Modifier
					.fillMaxWidth()
					.clickable { onRegionClick() }
					.padding(16.dp),
				verticalAlignment = Alignment.CenterVertically,
			) {
				Column(modifier = Modifier.weight(1f)) {
					Row(verticalAlignment = Alignment.CenterVertically) {
						SettingsTitle(stringResource(R.string.geo_exclusion_china))
						Box(
							modifier = Modifier
								.padding(horizontal = 6.dp)
								.size(4.dp)
								.background(
									color = MaterialTheme.colorScheme.onPrimaryContainer,
									shape = CircleShape,
								),
						)
						Text(
							text = stringResource(R.string.geo_exclusion_ranges_count, "2,847"),
							style = MaterialTheme.typography.labelLarge,
							color = MaterialTheme.colorScheme.onPrimaryContainer,
						)
					}
					Text(
						text = stringResource(R.string.geo_exclusion_last_updated, "12 Jun 2026"),
						style = MaterialTheme.typography.bodyMedium,
						color = MaterialTheme.colorScheme.onBackground,
						modifier = Modifier.padding(top = 4.dp),
					)
				}
				Icon(
					Icons.AutoMirrored.Filled.KeyboardArrowRight,
					stringResource(R.string.go),
					modifier = Modifier.size(iconSize),
					tint = MaterialTheme.colorScheme.onBackground.copy(alpha = 0.3f),
				)
			}
			HorizontalDivider(
				modifier = Modifier.fillMaxWidth(),
				thickness = 1.dp,
				color = MaterialTheme.colorScheme.outlineVariant,
			)
			Row(
				modifier = Modifier
					.fillMaxWidth()
					.clickable { onAddRegionClick() }
					.padding(16.dp),
				verticalAlignment = Alignment.CenterVertically,
			) {
				Box(
					modifier = Modifier
						.size(18.dp)
						.border(
							width = 1.dp,
							color = MaterialTheme.colorScheme.onBackground.copy(alpha = 0.3f),
							shape = CircleShape,
						),
					contentAlignment = Alignment.Center,
				) {
					Icon(
						imageVector = Icons.Default.Add,
						contentDescription = stringResource(R.string.geo_exclusion_add_region_button),
						tint = MaterialTheme.colorScheme.onBackground.copy(alpha = 0.3f),
						modifier = Modifier.size(12.dp),
					)
				}
				Spacer(modifier = Modifier.width(12.dp))
				Text(
					text = stringResource(R.string.geo_exclusion_add_region_button),
					style = MaterialTheme.typography.bodyMedium,
					color = MaterialTheme.colorScheme.onBackground.copy(alpha = 0.3f),
				)
			}
		}
	}
}
