package net.nymtech.vpn.tunnel

import android.os.Parcelable
import kotlinx.parcelize.Parcelize

@Parcelize
data class ErrorState(val cause: ErrorStateCause, val isBlocking: Boolean) : Parcelable
