package net.nymtech.nymvpn.ui.screens.settings.legal.licenses

import kotlinx.serialization.json.Json
import okio.BufferedSource

object LicenseParser {
    fun decode(source: BufferedSource): List<Artifact> {
        return Json.decodeFromString(source.readString(Charsets.UTF_8))
    }
}