package net.nymtech.nymvpn.ui.screens.settings.components

import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import net.nymtech.nymvpn.ui.common.buttons.surface.SelectionItem
import net.nymtech.nymvpn.ui.common.buttons.surface.SurfaceSelectionGroupButton

@Composable
fun SettingsGroup(items: List<SelectionItem>, modifier: Modifier = Modifier, background: Color = MaterialTheme.colorScheme.primaryContainer) {
	SurfaceSelectionGroupButton(
		items = items,
		background = background,
		modifier = modifier,
	)
}

/** Consecutive rows share one [SettingsGroup]; blocks are drawn on their own. */
@Composable
fun SettingsEntryList(entries: List<SettingsEntry>) {
	var index = 0
	while (index < entries.size) {
		when (val entry = entries[index]) {
			is SettingsBlock -> {
				entry.content()
				index++
			}
			is SettingsRow -> {
				val rows = entries.drop(index).takeWhile { it is SettingsRow }.filterIsInstance<SettingsRow>()
				SettingsGroup(items = rows.map { it.item })
				index += rows.size
			}
		}
	}
}
