package net.nymtech.vpn.model

import nym_vpn_lib.EntryPoint
import nym_vpn_lib.ExitPoint

data class ClientState(
    val vpnState: VpnState = VpnState.Down,
    val statistics: VpnStatistics = VpnStatistics(),
    val errorState: ErrorState = ErrorState.None,
    val mode: VpnMode = VpnMode.TWO_HOP_MIXNET,
    val entryPoint: EntryPoint? = null,
    val exitPoint: ExitPoint? = null,
)