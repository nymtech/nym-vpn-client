package net.nymtech.nymvpn.service.gateway.model

import androidx.annotation.Keep
import com.squareup.moshi.Json
import com.squareup.moshi.JsonClass


@Keep
@JsonClass(generateAdapter = true)
data class DescribedGateway(
    @Json(name = "bond") val bond: Bond,
    @Json(name = "self_described") val selfDescribed: SelfDescribed,
    @Json(name = "network_requester") val networkRequester: String?,
)
