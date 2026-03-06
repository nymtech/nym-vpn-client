package net.nymtech.vpn.model.connect

import nym_vpn_lib_types.MixnetTrafficConfig

/**
 * Parameters required to initialize VPN core.
 */
data class ConnectInitRequest(val mixnetParamConfig: MixnetTrafficConfig? = null)
