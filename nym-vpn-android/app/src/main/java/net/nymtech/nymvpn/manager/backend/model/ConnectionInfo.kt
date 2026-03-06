package net.nymtech.nymvpn.manager.backend.model

import nym_vpn_lib_types.ConnectionData
import nym_vpn_lib_types.EstablishConnectionData
import nym_vpn_lib_types.GatewayLightInfo
import nym_vpn_lib_types.OffsetDateTime
import nym_vpn_lib_types.TunnelConnectionData

data class ConnectionInfo(var entryGateway: GatewayLightInfo, var exitGateway: GatewayLightInfo, var connectedAt: OffsetDateTime?, var tunnel: TunnelConnectionData?)

fun EstablishConnectionData.toInfo(): ConnectionInfo = ConnectionInfo(this.entryGateway, this.exitGateway, null, this.tunnel)

fun ConnectionData.toInfo(): ConnectionInfo = ConnectionInfo(this.entryGateway, this.exitGateway, this.connectedAt, this.tunnel)
