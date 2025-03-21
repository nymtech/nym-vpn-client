package net.nymtech.nymvpn.ui.screens.settings.logs.components

import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import net.nymtech.logcatutil.model.LogMessage
import net.nymtech.nymvpn.ui.common.labels.LogTypeLabel

@Composable
fun LogRow(log: LogMessage, onClick: () -> Unit) {
	Row(
		horizontalArrangement = Arrangement.spacedBy(5.dp, Alignment.Start),
		verticalAlignment = Alignment.Top,
		modifier = Modifier
			.fillMaxSize()
			.clickable(
				interactionSource = remember { MutableInteractionSource() },
				indication = null,
				onClick = onClick,
			),
	) {
		Text(
			text = log.tag,
			modifier = Modifier.fillMaxSize(0.3f),
			style = MaterialTheme.typography.labelSmall,
		)
		LogTypeLabel(color = Color(log.level.color())) {
			Text(
				text = log.level.signifier,
				textAlign = TextAlign.Center,
				style = MaterialTheme.typography.labelSmall,
			)
		}
		Text(
			text = "${log.message} - ${log.time}",
			color = MaterialTheme.colorScheme.onBackground,
			style = MaterialTheme.typography.labelSmall,
		)
	}
}
