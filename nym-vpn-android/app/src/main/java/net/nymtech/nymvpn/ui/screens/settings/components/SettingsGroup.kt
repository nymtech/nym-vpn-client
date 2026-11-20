package net.nymtech.nymvpn.ui.screens.settings.components

import androidx.annotation.StringRes
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import net.nymtech.nymvpn.ui.common.buttons.surface.SelectionItem
import net.nymtech.nymvpn.ui.common.buttons.surface.SurfaceSelectionGroupButton
import java.text.Normalizer

@Composable
fun SettingsGroup(items: List<SelectionItem>, modifier: Modifier = Modifier, background: Color = MaterialTheme.colorScheme.primaryContainer) {
	SurfaceSelectionGroupButton(
		items = items,
		background = background,
		modifier = modifier,
	)
}

/** A settings row with the plain title used by search and the section it belongs to. */
data class SettingsEntry(@StringRes val section: Int, val title: String, val item: SelectionItem)

fun MutableList<SettingsEntry>.add(@StringRes section: Int, title: String, item: SelectionItem) = add(SettingsEntry(section, title, item))

private val combiningMarks = Regex("\\p{M}+")

private fun foldForSearch(text: String) = Normalizer.normalize(text, Normalizer.Form.NFD).replace(combiningMarks, "").lowercase()

/** True when [title] contains [query], ignoring case and accents. */
fun matchesSearch(title: String, query: String): Boolean = foldForSearch(title).contains(foldForSearch(query.trim()))
