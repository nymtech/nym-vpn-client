package net.nymtech.vpnclient.tunnel

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

@Parcelize
enum class ActionAfterDisconnect : Parcelable {
	Nothing,
	Block,
	Reconnect,
}
