package net.nymtech.nymvpn.ui.common.buttons

import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.ripple
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp

@Composable
fun SelectionItemButton(leading: (@Composable () -> Unit)? = null, buttonText: String, trailing: (@Composable () -> Unit)? = null, onClick: () -> Unit, ripple: Boolean = true) {
	Row(
		verticalAlignment = Alignment.CenterVertically,
		horizontalArrangement = Arrangement.Start,
		modifier = Modifier
			.fillMaxSize()
			.padding(vertical = 16.dp)
			.clickable(
				indication = if (ripple) ripple() else null,
				interactionSource = remember { MutableInteractionSource() },
				onClick = { onClick() },
			),
	) {
		leading?.let {
			it()
		}
		Text(
			buttonText,
			style = MaterialTheme.typography.bodyLarge,
			color = MaterialTheme.colorScheme.onPrimaryContainer,
		)
		trailing?.let {
			it()
		}
	}
}
