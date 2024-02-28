package net.nymtech.nymvpn.ui.screens.settings.legal.licenses

import kotlinx.serialization.Serializable

@Serializable
data class SpdxLicenses(val identifier: String, val name: String, val url: String)

@Serializable
data class Scm(val url: String)

@Serializable
data class UnknownLicenses(val name: String, val url: String)

@Serializable
data class Artifact(
    val groupId: String,
    val artifactId: String,
    val version: String,
    val name: String? = null,
    val spdxLicenses: List<SpdxLicenses>? = null,
    val scm: Scm? = null,
    val unknownLicenses: List<UnknownLicenses>? = null,
)