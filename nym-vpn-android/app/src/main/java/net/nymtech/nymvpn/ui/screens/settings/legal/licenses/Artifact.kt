package net.nymtech.nymvpn.ui.screens.settings.legal.licenses

import kotlinx.serialization.Serializable
import kotlinx.serialization.json.Json
import net.nymtech.vpn.model.License

@Serializable
data class SpdxLicenses(val identifier: String, val name: String, val url: String)

@Serializable
data class Scm(val url: String)

@Serializable
data class UnknownLicenses(val name: String, val url: String)

@Serializable
data class Artifact(
	val groupId: String? = null,
	val artifactId: String? = null,
	val version: String? = null,
	val name: String? = null,
	val spdxLicenses: List<SpdxLicenses>? = null,
	val scm: Scm? = null,
	val unknownLicenses: List<UnknownLicenses>? = null,
) {
	companion object {
		fun fromJsonList(text: String): Result<List<Artifact>> {
			return kotlin.runCatching {
				Json.decodeFromString<List<Artifact>>(text)
					.distinctBy { artifact -> artifact.name }
			}
		}
		fun from(license: License): Artifact {
			return Artifact(
				version = license.version,
				name = "${license.name} (Rust)",
				unknownLicenses = license.license?.let { listOf(UnknownLicenses(it, "")) },
				scm = license.repository?.let { Scm(it) },
			)
		}
		fun from(licenses: List<License>): List<Artifact> {
			return licenses.map { from(it) }
		}
	}
}
