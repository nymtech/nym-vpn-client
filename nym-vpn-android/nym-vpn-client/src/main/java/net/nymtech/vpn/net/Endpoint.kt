package net.nymtech.vpn.net

import android.os.Parcelable
import kotlinx.parcelize.Parcelize
import java.net.InetSocketAddress

@Parcelize
data class Endpoint(val address: InetSocketAddress, val protocol: TransportProtocol) : Parcelable
