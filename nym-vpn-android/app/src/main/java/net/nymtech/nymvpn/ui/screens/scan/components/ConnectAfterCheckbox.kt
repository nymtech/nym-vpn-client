package net.nymtech.nymvpn.ui.screens.scan.components

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.selection.toggleable
import androidx.compose.material3.Checkbox
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.semantics.Role
import androidx.compose.ui.unit.dp

@Composable
fun ConnectAfterCheckbox(
	checked: Boolean,
	onCheckedChange: (Boolean) -> Unit,
	label: String,
	modifier: Modifier = Modifier,
	enabled: Boolean = true,
	textColor: Color = MaterialTheme.colorScheme.onSurface,
) {
	Row(
		verticalAlignment = Alignment.CenterVertically,
		horizontalArrangement = Arrangement.spacedBy(12.dp),
		modifier = modifier.toggleable(value = checked, enabled = enabled, role = Role.Checkbox, onValueChange = onCheckedChange),
	) {
		Checkbox(
			checked = checked,
			onCheckedChange = null,
			enabled = enabled,
			modifier = Modifier.size(20.dp),
		)
		Text(
			text = label,
			style = MaterialTheme.typography.bodyMedium,
			color = textColor,
		)
	}
}
