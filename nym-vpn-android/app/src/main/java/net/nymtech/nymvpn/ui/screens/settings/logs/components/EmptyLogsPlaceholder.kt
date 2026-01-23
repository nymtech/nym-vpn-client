package net.nymtech.nymvpn.ui.screens.settings.logs.components

import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextAlign
import net.nymtech.nymvpn.R

@Composable
fun EmptyLogsPlaceholder() {
	Box(
		modifier = Modifier.fillMaxSize(),
		contentAlignment = Alignment.Center,
	) {
		Text(
			text = stringResource(R.string.logs_empty_placeholder),
			style = MaterialTheme.typography.bodyMedium,
			color = MaterialTheme.colorScheme.outline,
			textAlign = TextAlign.Center,
		)
	}
}
