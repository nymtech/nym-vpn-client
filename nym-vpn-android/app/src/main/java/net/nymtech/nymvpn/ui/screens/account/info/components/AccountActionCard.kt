package net.nymtech.nymvpn.ui.screens.account.info.components

import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.IntrinsicSize
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.outlined.Launch
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
fun AccountActionCard(title: String, subtitle: @Composable (() -> Unit)? = null, icon: ImageVector, onClick: () -> Unit) {
	val interactionSource = remember { MutableInteractionSource() }

	Card(
		modifier = Modifier.fillMaxWidth(),
		shape = RoundedCornerShape(8.dp),
		colors = CardDefaults.cardColors(containerColor = MaterialTheme.colorScheme.primaryContainer),
	) {
		Box(
			contentAlignment = Alignment.Center,
			modifier = Modifier
				.clickable(
					interactionSource = interactionSource,
					indication = null,
				) { onClick() }
				.fillMaxWidth()
				.height(IntrinsicSize.Min),
		) {
			Row(
				verticalAlignment = Alignment.CenterVertically,
				modifier = Modifier.fillMaxSize(),
			) {
				Row(
					verticalAlignment = Alignment.CenterVertically,
					modifier = Modifier
						.weight(1f)
						.fillMaxSize()
						.padding(end = 4.dp.scaledWidth()),
				) {
					Box(modifier = Modifier.padding(start = 16.dp.scaledWidth()))
					Box(modifier = Modifier.padding(end = 16.dp.scaledWidth())) {
						Icon(
							imageVector = icon,
							contentDescription = null,
							modifier = Modifier.size(24.dp.scaledWidth()),
							tint = MaterialTheme.colorScheme.onSurfaceVariant,
						)
					}
					Column(
						horizontalAlignment = Alignment.Start,
						verticalArrangement = Arrangement.spacedBy(2.dp, Alignment.CenterVertically),
						modifier = Modifier
							.fillMaxWidth()
							.padding(vertical = 16.dp.scaledHeight()),
					) {
						Text(
							title,
							style = MaterialTheme.typography.titleSmall,
							color = MaterialTheme.colorScheme.onPrimaryContainer,
						)
						subtitle?.invoke()
					}
				}
				Box(
					contentAlignment = Alignment.CenterEnd,
					modifier = Modifier
						.weight(0.2f)
						.padding(horizontal = 16.dp.scaledWidth()),
				) {
					Icon(
						imageVector = Icons.AutoMirrored.Outlined.Launch,
						contentDescription = stringResource(R.string.go),
						modifier = Modifier.size(24.dp),
						tint = MaterialTheme.colorScheme.outline,
					)
				}
			}
		}
	}
}
