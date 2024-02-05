package net.nymtech.vpn_client.tun_provider

import java.net.InetAddress

data class TunConfig(
    val addresses: ArrayList<InetAddress>,
    val dnsServers: ArrayList<InetAddress>,
    val routes: ArrayList<InetNetwork>,
    val mtu: Int
)