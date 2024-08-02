package net.nymtech.vpn.model

import kotlinx.serialization.SerialName
import kotlinx.serialization.Serializable
import kotlinx.serialization.json.Json
import timber.log.Timber

@Serializable
data class License(
	val name: String,
	val version: String,
	val authors: String?,
	val repository: String?,
	val license: String?,
	@SerialName("license_file") val licenseFile: String?,
	val description: String?,
) {
	companion object {
		fun fromJsonList(text: String): Result<List<License>> {
			return kotlin.runCatching {
				Json.decodeFromString<List<License>>(text)
					.distinctBy { it.name }
			}.onFailure {
				Timber.e(it)
			}
		}
	}
}
