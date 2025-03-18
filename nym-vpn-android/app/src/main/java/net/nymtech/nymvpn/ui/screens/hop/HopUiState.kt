package net.nymtech.nymvpn.ui.screens.hop

import net.nymtech.vpn.model.NymGateway
import java.util.*

data class HopUiState(
	val error: Boolean = false,
	val query: String = "",
	val countries: List<Locale> = emptyList(),
	val queriedGateways: List<NymGateway> = emptyList(),
)
