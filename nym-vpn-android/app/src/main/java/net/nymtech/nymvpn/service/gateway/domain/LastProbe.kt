package net.nymtech.nymvpn.service.gateway.domain

import androidx.annotation.Keep
import com.squareup.moshi.Json
import com.squareup.moshi.JsonClass

@Keep
@JsonClass(generateAdapter = true)
data class LastProbe(
	@Json(name = "last_updated_utc") val lastUpdatedUTC: String,
	@Json(name = "outcome") val outcome: Outcome,
)
