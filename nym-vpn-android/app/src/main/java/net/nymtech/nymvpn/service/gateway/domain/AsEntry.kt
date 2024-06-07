package net.nymtech.nymvpn.service.gateway.domain

import androidx.annotation.Keep
import com.squareup.moshi.Json
import com.squareup.moshi.JsonClass

@Keep
@JsonClass(generateAdapter = true)
data class AsEntry(
	@Json(name = "can_connect") val canConnect: Boolean,
	@Json(name = "can_route") val canRoute: Boolean,
)
