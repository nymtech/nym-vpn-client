package net.nymtech.nymvpn.service.gateway.model

import androidx.annotation.Keep
import com.squareup.moshi.Json
import com.squareup.moshi.JsonClass

@Keep
@JsonClass(generateAdapter = true)
data class Keys(
	@Json(name = "ed25519") val ed25519: String,
	@Json(name = "x25519") val x25519: String,
)
