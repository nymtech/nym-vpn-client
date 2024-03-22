package net.nymtech.nymvpn.ui.screens.hop

import net.nymtech.nymvpn.ui.HopType
import net.nymtech.vpn.model.Hop
import net.nymtech.vpn.model.HopCountries

data class HopUiState(
    val countries: HopCountries = emptySet(),
    val hopType: HopType = HopType.FIRST,
    val queriedCountries: HopCountries = emptySet(),
    val selected: Hop.Country? = null,
    val query: String = ""
)