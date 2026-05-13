package net.nymtech.nymvpn.ui.common.labels

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import net.nymtech.nymvpn.R

@Composable
fun SelectedLabel() {
	Row(
		modifier = Modifier.fillMaxWidth(),
		horizontalArrangement = Arrangement.End,
		verticalAlignment = Alignment.CenterVertically,
	) {
		Text(
			stringResource(id = R.string.is_selected),
			color =
			MaterialTheme.colorScheme.primary,
			style = MaterialTheme.typography.labelSmall,
		)
	}
}
