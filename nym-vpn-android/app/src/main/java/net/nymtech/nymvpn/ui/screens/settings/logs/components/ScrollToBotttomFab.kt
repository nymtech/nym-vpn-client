package net.nymtech.nymvpn.ui.screens.settings.logs.components

import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.KeyboardDoubleArrowDown
import androidx.compose.material3.FloatingActionButton
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R

@Composable
fun ScrollToBottomFab(onClick: () -> Unit) {
	FloatingActionButton(
		onClick = onClick,
		shape = RoundedCornerShape(16.dp),
		containerColor = MaterialTheme.colorScheme.primary,
	) {
		Icon(
			imageVector = Icons.Filled.KeyboardDoubleArrowDown,
			contentDescription = stringResource(R.string.scroll_down),
			tint = MaterialTheme.colorScheme.onPrimary,
		)
	}
}
