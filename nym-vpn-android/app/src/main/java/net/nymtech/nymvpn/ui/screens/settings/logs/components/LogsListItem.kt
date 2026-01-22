package net.nymtech.nymvpn.ui.screens.settings.logs.components

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Close
import androidx.compose.material.icons.filled.Info
import androidx.compose.material.icons.filled.Warning
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.unit.dp
import net.nymtech.logcatutil.model.LogMessage

@Composable
fun LogsListItem(log: LogMessage, modifier: Modifier = Modifier) {
	val (icon, tint) = when (log.level.signifier.uppercase()) {
		"E", "A" -> {
			Icons.Filled.Close to MaterialTheme.colorScheme.error
		}
		"W" -> {
			Icons.Filled.Warning to Color(0xFFFFC107)
		}

		else -> {
			Icons.Filled.Info to Color(0xFF7C4DFF)
		}
	}

	Row(
		horizontalArrangement = Arrangement.spacedBy(12.dp),
		verticalAlignment = Alignment.Top,
		modifier = modifier,
	) {
		Icon(
			imageVector = icon,
			contentDescription = null,
			tint = tint,
			modifier = Modifier.size(18.dp),
		)

		Column(verticalArrangement = Arrangement.spacedBy(4.dp)) {
			Text(
				text = log.message,
				style = MaterialTheme.typography.bodyMedium,
				color = MaterialTheme.colorScheme.onBackground,
			)
			Text(
				text = log.time,
				style = MaterialTheme.typography.bodySmall,
				color = MaterialTheme.colorScheme.onBackground.copy(alpha = 0.6f),
			)
		}
	}
}
