package net.nymtech.vpn_client.net

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

@Parcelize
enum class TransportProtocol : Parcelable {
    Tcp, Udp
}