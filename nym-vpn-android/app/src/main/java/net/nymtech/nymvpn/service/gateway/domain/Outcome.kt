package net.nymtech.nymvpn.service.gateway.domain

import androidx.annotation.Keep
import com.squareup.moshi.JsonClass

@Keep
@JsonClass(generateAdapter = true)
data class Outcome(
	val asEntry: AsEntry,
	val asExit: AsExit,
)
