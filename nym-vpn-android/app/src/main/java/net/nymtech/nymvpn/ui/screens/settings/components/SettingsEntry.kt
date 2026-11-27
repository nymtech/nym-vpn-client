package net.nymtech.nymvpn.ui.screens.settings.components

import androidx.compose.runtime.Composable
import net.nymtech.nymvpn.ui.common.buttons.surface.SelectionItem

/** A searchable item on the settings screen; [description] is null when absent or not plain text. */
sealed interface SettingsEntry {
	val section: SettingsSection
	val title: String
	val description: String?
}

class SettingsRow(override val section: SettingsSection, override val title: String, override val description: String? = null, val item: SelectionItem) : SettingsEntry

class SettingsBlock(override val section: SettingsSection, override val title: String, override val description: String? = null, val content: @Composable () -> Unit) : SettingsEntry
