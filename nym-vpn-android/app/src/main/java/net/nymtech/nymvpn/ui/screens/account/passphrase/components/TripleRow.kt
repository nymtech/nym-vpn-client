package net.nymtech.nymvpn.ui.screens.account.passphrase.components

import androidx.compose.foundation.Canvas
import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Check
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.geometry.Offset
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.StrokeCap
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.ui.screens.account.passphrase.modal.InfoItem

@Composable
fun TripleRow(items: List<InfoItem>, spacing: Dp = 16.dp) {
	Row(
		modifier = Modifier
			.fillMaxWidth(),
		horizontalArrangement = Arrangement.spacedBy(spacing),
		verticalAlignment = Alignment.Top,
	) {
		items.forEach { item ->
			Column(
				modifier = Modifier.weight(1f),
				horizontalAlignment = Alignment.CenterHorizontally,
			) {
				Box(
					contentAlignment = Alignment.Center,
					modifier = Modifier.size(28.dp),
				) {
					Icon(
						imageVector = item.icon,
						contentDescription = null,
						tint = MaterialTheme.colorScheme.onPrimaryContainer,
						modifier = Modifier.size(22.dp),
					)
					if (item.negative) {
						val bgColor = MaterialTheme.colorScheme.errorContainer
						Canvas(modifier = Modifier.size(34.dp)) {
							drawLine(
								color = bgColor,
								start = Offset(12f, size.height - 12f),
								end = Offset(size.width - 12f, 12f),
								strokeWidth = 6f,
								cap = StrokeCap.Round,
							)
						}
					} else {
						Box(
							modifier = Modifier
								.align(Alignment.BottomStart)
								.offset(x = 16.dp, y = 0.dp)
								.size(12.dp)
								.clip(CircleShape)
								.background(Color(0xFF66BB6A)),
							contentAlignment = Alignment.Center,
						) {
							Box(
								modifier = Modifier
									.size(12.dp)
									.clip(CircleShape)
									.background(MaterialTheme.colorScheme.primary),
								contentAlignment = Alignment.Center,
							) {
								Icon(
									imageVector = Icons.Filled.Check,
									contentDescription = null,
									tint = MaterialTheme.colorScheme.onPrimary,
									modifier = Modifier.size(10.dp),
								)
							}
						}
					}
				}
				Text(
					text = item.label,
					color = MaterialTheme.colorScheme.onBackground,
					style = MaterialTheme.typography.bodySmall,
					textAlign = TextAlign.Center,
					modifier = Modifier.padding(top = 4.dp),
				)
			}
		}
	}
}
