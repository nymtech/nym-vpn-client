package net.nymtech.vpn.model

data class ClientState(
    val vpnState: VpnState = VpnState.Down,
    val statistics: VpnStatistics = VpnStatistics(),
    val errorState: ErrorState = ErrorState.None,
    val mode: VpnMode = VpnMode.TWO_HOP_MIXNET
)