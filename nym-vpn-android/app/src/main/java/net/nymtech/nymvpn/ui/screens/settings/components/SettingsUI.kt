package net.nymtech.nymvpn.ui.screens.settings.components

import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.KeyboardArrowRight
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.res.stringResource
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.theme.iconSize
import net.nymtech.nymvpn.util.extensions.scaledWidth

@Composable
fun SettingsArrowIcon() {
	Icon(
		Icons.AutoMirrored.Filled.KeyboardArrowRight,
		stringResource(R.string.go),
		modifier = Modifier.size(iconSize),
		tint = MaterialTheme.colorScheme.onBackground,
	)
}

@Composable
fun SettingsTitle(title: String) {
	Text(
		title,
		style = MaterialTheme.typography.bodyLarge,
		color = MaterialTheme.colorScheme.onPrimaryContainer,
	)
}

@Composable
fun SettingsIcon(icon: ImageVector, description: String) {
	Icon(
		icon,
		description,
		modifier = Modifier.size(iconSize.scaledWidth()),
		tint = MaterialTheme.colorScheme.onSurfaceVariant,
	)
}
