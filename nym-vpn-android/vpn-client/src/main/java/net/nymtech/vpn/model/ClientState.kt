package net.nymtech.vpn.model

data class ClientState(
    val vpnState: VpnState = VpnState.DOWN,
    val statistics: VpnStatistics = VpnStatistics(),
    val errorState: ErrorState = ErrorState.None
)