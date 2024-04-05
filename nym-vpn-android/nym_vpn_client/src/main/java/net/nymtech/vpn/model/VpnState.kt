package net.nymtech.vpn.model

sealed class VpnState {
    data object Up : VpnState()
    data object Down : VpnState()
    data object Connecting {
        data object InitializingClient : VpnState()
        data object EstablishingConnection : VpnState()
    }

    data object Disconnecting : VpnState()
}