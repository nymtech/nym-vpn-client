package net.nymtech.nymvpn.service.gateway.model

import androidx.annotation.Keep
import com.squareup.moshi.Json
import com.squareup.moshi.JsonClass

@Keep
@JsonClass(generateAdapter = true)
data class MixnetWebsockets(
	@Json(name = "ws_port") val wsPort: Int?,
	@Json(name = "wss_port") val wssPort: Int?,
)
