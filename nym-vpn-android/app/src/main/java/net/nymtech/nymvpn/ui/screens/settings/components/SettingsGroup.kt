package net.nymtech.nymvpn.ui.screens.settings.components

import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import net.nymtech.nymvpn.ui.common.buttons.surface.SelectionItem
import net.nymtech.nymvpn.ui.common.buttons.surface.SurfaceSelectionGroupButton

@Composable
fun SettingsGroup(items: List<SelectionItem>, modifier: Modifier = Modifier, background: Color = MaterialTheme.colorScheme.surface) {
	SurfaceSelectionGroupButton(
		items = items,
		background = background,
		modifier = modifier,
	)
}
