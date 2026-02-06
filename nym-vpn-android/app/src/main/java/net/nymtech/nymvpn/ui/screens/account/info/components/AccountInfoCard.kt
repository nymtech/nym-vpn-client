package net.nymtech.nymvpn.ui.screens.account.info.components

import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.ContentCopy
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.util.extensions.scaledHeight
import net.nymtech.nymvpn.util.extensions.scaledWidth

@Composable
fun AccountInfoCard(title: String, value: String, icon: ImageVector, onClick: () -> Unit) {
	val interactionSource = remember { MutableInteractionSource() }

	Card(
		modifier = Modifier.fillMaxWidth(),
		shape = RoundedCornerShape(8.dp),
		colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.surface),
	) {
		Box(
			contentAlignment = Alignment.Center,
			modifier = Modifier
				.clickable(
					interactionSource = interactionSource,
					indication = null,
				) { onClick() }
				.fillMaxWidth(),
		) {
			Row(
				verticalAlignment = Alignment.CenterVertically,
				modifier = Modifier
					.fillMaxWidth()
					.padding(horizontal = 16.dp),
			) {
				Row(
					verticalAlignment = Alignment.CenterVertically,
					modifier = Modifier.weight(1f),
				) {
					Column(
						horizontalAlignment = Alignment.Start,
						modifier = Modifier
							.fillMaxWidth()
							.padding(vertical = 16.dp.scaledHeight()),
					) {
						Row(verticalAlignment = Alignment.CenterVertically) {
							Icon(
								imageVector = icon,
								contentDescription = null,
								modifier = Modifier.size(24.dp.scaledWidth()),
								tint = MaterialTheme.colorScheme.outline,
							)
							Text(
								text = title,
								style = MaterialTheme.typography.bodyLarge.copy(MaterialTheme.colorScheme.onSurface),
								modifier = Modifier.padding(start = 10.dp),
							)
						}
						Text(
							text = value,
							style = MaterialTheme.typography.bodyMedium.copy(MaterialTheme.colorScheme.outline),
							modifier = Modifier.padding(top = 10.dp),
						)
					}
				}
				Box(
					contentAlignment = Alignment.CenterEnd,
					modifier = Modifier.padding(start = 16.dp.scaledWidth()),
				) {
					Icon(
						imageVector = Icons.Outlined.ContentCopy,
						contentDescription = stringResource(R.string.go),
						modifier = Modifier.size(20.dp),
						tint = MaterialTheme.colorScheme.onBackground,
					)
				}
			}
		}
	}
}
