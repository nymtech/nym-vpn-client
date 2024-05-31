package net.nymtech.nymvpn.service.gateway.domain

import androidx.annotation.Keep
import com.squareup.moshi.Json
import com.squareup.moshi.JsonClass

@Keep
@JsonClass(generateAdapter = true)
data class Gateway(
	@Json(name = "identity_key") val identityKey: String,
	@Json(name = "location") val location: Location,
	@Json(name = "last_probe") val lastProbe: LastProbe,
)
