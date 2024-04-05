package net.nymtech.nymvpn.service.gateway.model

import androidx.annotation.Keep
import com.squareup.moshi.Json
import com.squareup.moshi.JsonClass

@Keep
@JsonClass(generateAdapter = true)
data class BuildInformation(
    @Json(name = "binary_name") val binaryName: String,
    @Json(name = "build_timestamp") val buildTimestamp: String,
    @Json(name = "build_version") val buildVersion: String,
    @Json(name = "commit_sha") val commitSha: String,
    @Json(name = "commit_timestamp") val commitTimestamp: String,
    @Json(name = "commit_branch") val commitBranch: String,
    @Json(name = "rustc_version") val rustcVersion: String,
    @Json(name = "rustc_channel") val rustcChannel: String,
    @Json(name = "cargo_profile") val cargoProfile: String
)