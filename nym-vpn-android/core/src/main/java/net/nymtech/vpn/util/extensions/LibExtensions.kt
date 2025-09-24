package net.nymtech.vpn.util.extensions

import net.nymtech.vpn.backend.Tunnel
import net.nymtech.vpn.util.Base58
import nym_vpn_lib_types.EntryPoint
import nym_vpn_lib_types.ExitPoint
import nym_vpn_lib_types.TunnelEvent
import nym_vpn_lib_types.TunnelState

fun TunnelEvent.NewState.asTunnelState(): Tunnel.State {
	return when (this.v1) {
		is TunnelState.Connected -> Tunnel.State.Up
		is TunnelState.Connecting -> Tunnel.State.EstablishingConnection
		TunnelState.Disconnected -> Tunnel.State.Down
		is TunnelState.Disconnecting -> Tunnel.State.Disconnecting
		is TunnelState.Error -> Tunnel.State.Down
		is TunnelState.Offline -> Tunnel.State.Offline
	}
}

fun EntryPoint.asString(): String {
	return when (val entry = this) {
		is EntryPoint.Gateway -> entry.identity
		is EntryPoint.Country -> entry.twoLetterIsoCountryCode.lowercase()
		EntryPoint.Random -> "random"
		is EntryPoint.Region -> entry.region.lowercase()
	}
}

fun ExitPoint.asString(): String {
	return when (val exit = this) {
		is ExitPoint.Gateway -> exit.identity
		is ExitPoint.Country -> exit.twoLetterIsoCountryCode.lowercase()
		is ExitPoint.Address -> exit.address
		is ExitPoint.Random -> "random"
		is ExitPoint.Region -> exit.region
	}
}

fun String.asEntryPoint(): EntryPoint {
	return when {
		this == "random" -> EntryPoint.Random
		length == 2 -> EntryPoint.Country(this.uppercase())
		Base58.isValidBase58(this, 32) -> EntryPoint.Gateway(this)
		else -> EntryPoint.Random
	}
}

fun String.asExitPoint(): ExitPoint {
	return when {
		length == 2 -> ExitPoint.Country(this.uppercase())
		length == 134 -> ExitPoint.Address(this)
		Base58.isValidBase58(this, 32) -> ExitPoint.Gateway(this)
		else -> throw IllegalArgumentException("Invalid exit id")
	}
}
