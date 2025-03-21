package net.nymtech.nymvpn.ui.screens.permission.components

import androidx.compose.foundation.background
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.width
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.util.extensions.scaledWidth

@Composable
fun PermissionHeader() {
	Row(horizontalArrangement = Arrangement.spacedBy(16.dp.scaledWidth())) {
		Box(
			modifier = Modifier
				.width(2.dp)
				.height(60.dp)
				.background(color = MaterialTheme.colorScheme.primary, shape = RoundedCornerShape(size = 4.dp)),
		)
		Text(
			stringResource(R.string.permission_message),
			style = MaterialTheme.typography.bodyMedium,
			color = MaterialTheme.colorScheme.onSurfaceVariant,
		)
	}
}
