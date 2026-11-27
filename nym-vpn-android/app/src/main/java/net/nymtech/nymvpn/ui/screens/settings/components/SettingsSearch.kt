package net.nymtech.nymvpn.ui.screens.settings.components

import java.text.Normalizer

private val combiningMarks = Regex("\\p{M}+")

private fun foldForSearch(text: String) = Normalizer.normalize(text, Normalizer.Form.NFD).replace(combiningMarks, "").lowercase()

/** True when [text] contains [query], ignoring case and accents. */
fun matchesSearch(text: String, query: String): Boolean = foldForSearch(text).contains(foldForSearch(query))

private fun SettingsEntry.matchesSearch(query: String): Boolean = matchesSearch(title, query) || description?.let { matchesSearch(it, query) } == true

/** Matching entries per section, in display order; a section title match keeps the whole section. */
fun filterSettingsEntries(entries: List<SettingsEntry>, query: String, sectionTitle: (SettingsSection) -> String?): Map<SettingsSection, List<SettingsEntry>> {
	val search = query.trim()
	return buildMap {
		SettingsSection.entries.forEach { section ->
			val sectionEntries = entries.filter { it.section == section }
			val title = sectionTitle(section)
			val matches = if (search.isEmpty() || (title != null && matchesSearch(title, search))) {
				sectionEntries
			} else {
				sectionEntries.filter { it.matchesSearch(search) }
			}
			if (matches.isNotEmpty()) put(section, matches)
		}
	}
}
