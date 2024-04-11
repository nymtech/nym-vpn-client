package net.nymtech.nymvpn.service.gateway.model

import androidx.annotation.Keep
import com.squareup.moshi.Json
import com.squareup.moshi.JsonClass

@Keep
@JsonClass(generateAdapter = true)
data class Bond(
	@Json(name = "pledge_amount") val pledgeAmount: PledgeAmount,
	@Json(name = "owner") val owner: String,
	@Json(name = "block_height") val blockHeight: Int,
	@Json(name = "gateway") val gateway: Gateway,
	@Json(name = "proxy") val proxy: String?,
)
