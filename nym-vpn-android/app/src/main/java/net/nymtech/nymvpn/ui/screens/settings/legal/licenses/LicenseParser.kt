package net.nymtech.nymvpn.ui.screens.settings.legal.licenses

import kotlinx.serialization.SerializationException
import kotlinx.serialization.json.Json
import timber.log.Timber

object LicenseParser {
	fun decode(licenseJson: String): List<Artifact> {
		try {
			return Json.decodeFromString<List<Artifact>>(licenseJson)
				.distinctBy { it.name }
		} catch (e: SerializationException) {
			Timber.e(e)
		} catch (e: IllegalArgumentException) {
			Timber.e(e)
		}
		return emptyList()
	}
}
