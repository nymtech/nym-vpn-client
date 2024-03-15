package net.nymtech.nymvpn.ui.screens.hop

import net.nymtech.vpn.model.Hop
import net.nymtech.vpn.model.HopCountries

data class HopUiState(
    val loading: Boolean = true,
    val countries: HopCountries = emptyList(),
    val queriedCountries: HopCountries = emptyList(),
    val selected: Hop.Country? = null,
    val query: String = ""
)