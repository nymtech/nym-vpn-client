package net.nymtech.nymvpn.ui.common.buttons

import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.ripple.rememberRipple
import androidx.compose.material3.Card
import androidx.compose.material3.CardDefaults
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.util.scaledHeight

@Composable
fun SelectionItemButton(leading: @Composable () -> Unit, buttonText: String, trailing: (@Composable () -> Unit)? = null, onClick: () -> Unit) {
	Card(
		modifier =
		Modifier.clip(RoundedCornerShape(8.dp))
			.clickable(
				indication = rememberRipple(),
				interactionSource = remember { MutableInteractionSource() },
				onClick = { onClick() },
			)
			.height(56.dp.scaledHeight()),
		colors =
		CardDefaults.cardColors(
			containerColor = MaterialTheme.colorScheme.background,
		),
	) {
		Row(
			verticalAlignment = Alignment.CenterVertically,
			horizontalArrangement = Arrangement.Start,
			modifier = Modifier.fillMaxWidth(),
		) {
			leading()
			Text(
				buttonText,
				style = MaterialTheme.typography.bodyLarge,
				color = MaterialTheme.colorScheme.onSurface,
			)
			trailing?.let {
				it()
			}
		}
	}
}
