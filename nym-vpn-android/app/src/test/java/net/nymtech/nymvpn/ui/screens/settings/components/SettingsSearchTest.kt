package net.nymtech.nymvpn.ui.screens.settings.components

import net.nymtech.nymvpn.ui.common.buttons.surface.SelectionItem
import org.junit.Assert.assertEquals
import org.junit.Assert.assertFalse
import org.junit.Assert.assertTrue
import org.junit.Test

class SettingsSearchTest {

	private val sectionTitles = mapOf(
		SettingsSection.ACCOUNT to "Account",
		SettingsSection.PROTECTION to "Protection",
		SettingsSection.CONNECTION to "Connection",
		SettingsSection.ROUTING to "Routing",
		SettingsSection.APP to "App",
		SettingsSection.HELP_DATA to "Help and data",
		SettingsSection.FOOTER to null,
	)

	private val entries = listOf(
		block(SettingsSection.ACCOUNT, "Get started"),
		row(SettingsSection.ACCOUNT, "Redeem a code"),
		row(SettingsSection.PROTECTION, "Block ads", "Blocking ads and trackers"),
		row(SettingsSection.PROTECTION, "Kill switch"),
		row(SettingsSection.CONNECTION, "Customize DNS"),
		row(SettingsSection.ROUTING, "Split tunneling"),
		row(SettingsSection.ROUTING, "Geo-exclusion", "Specific IP ranges connect directly"),
		block(SettingsSection.FOOTER, "Log out"),
		row(SettingsSection.FOOTER, "Quit NymVPN"),
		block(SettingsSection.FOOTER, "App version: 2.2.2"),
	)

	private fun row(section: SettingsSection, title: String, description: String? = null) = SettingsRow(section = section, title = title, description = description, item = SelectionItem(title = {}))

	private fun block(section: SettingsSection, title: String) = SettingsBlock(section = section, title = title, content = {})

	private fun filter(query: String) = filterSettingsEntries(entries, query) { sectionTitles.getValue(it) }
		.mapValues { (_, sectionEntries) -> sectionEntries.map { it.title } }

	@Test
	fun `matches part of a text ignoring case`() {
		assertTrue(matchesSearch("Customize DNS", "dns"))
	}

	@Test
	fun `matches ignoring accents`() {
		assertTrue(matchesSearch("Écran de démarrage", "ecran"))
	}

	@Test
	fun `does not match unrelated text`() {
		assertFalse(matchesSearch("Kill switch", "zzz"))
	}

	@Test
	fun `blank query keeps every section that has entries, in display order`() {
		val result = filter("  ")
		assertEquals(
			listOf(SettingsSection.ACCOUNT, SettingsSection.PROTECTION, SettingsSection.CONNECTION, SettingsSection.ROUTING, SettingsSection.FOOTER),
			result.keys.toList(),
		)
		assertEquals(listOf("Log out", "Quit NymVPN", "App version: 2.2.2"), result[SettingsSection.FOOTER])
	}

	@Test
	fun `row title match keeps only that row under its section`() {
		assertEquals(mapOf(SettingsSection.CONNECTION to listOf("Customize DNS")), filter(" dns "))
	}

	@Test
	fun `row description match keeps that row`() {
		assertEquals(mapOf(SettingsSection.PROTECTION to listOf("Block ads")), filter("trackers"))
	}

	@Test
	fun `block title match keeps that block`() {
		assertEquals(mapOf(SettingsSection.ACCOUNT to listOf("Get started")), filter("get started"))
		assertEquals(mapOf(SettingsSection.FOOTER to listOf("Log out")), filter("log out"))
	}

	@Test
	fun `footer entries are found by their own titles`() {
		assertEquals(mapOf(SettingsSection.FOOTER to listOf("Quit NymVPN")), filter("quit"))
		assertEquals(mapOf(SettingsSection.FOOTER to listOf("App version: 2.2.2")), filter("version"))
	}

	@Test
	fun `section title match keeps the whole section`() {
		assertEquals(mapOf(SettingsSection.ROUTING to listOf("Split tunneling", "Geo-exclusion")), filter("routing"))
		assertEquals(mapOf(SettingsSection.ACCOUNT to listOf("Get started", "Redeem a code")), filter("account"))
	}

	@Test
	fun `no match returns no sections`() {
		assertTrue(filter("zzz").isEmpty())
	}
}
