package net.nymtech.nymvpn.ui.screens.settings.legal.licenses

import kotlinx.serialization.json.Json
import okio.BufferedSource

object LicenseParser {
	fun decode(source: BufferedSource): List<Artifact> {
		return Json.decodeFromString<List<Artifact>>(source.readString(Charsets.UTF_8))
			.distinctBy { it.name }
	}
}
