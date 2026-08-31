package net.nymtech.nymvpn.ui.screens.main

import net.nymtech.vpn.backend.Tunnel

/**
 * Decides what the main-screen connection timer should do for a tunnel state change.
 */
object ConnectionTimerPolicy {

	sealed interface Command {
		data class Start(val connectedAtSeconds: Long) : Command
		data object Stop : Command
		data object None : Command
	}

	// The timer is only shown while the tunnel is Up; the session start (connectedAt) is
	// preserved across offline gaps and reconnects, so on recovery the timer resumes with
	// the cumulative session time. Up may briefly be reported before the Connected event
	// delivers connectedAt — leave the timer untouched rather than restarting it.
	fun evaluate(tunnelState: Tunnel.State, connectedAtSeconds: Long?): Command = when (tunnelState) {
		is Tunnel.State.Up -> if (connectedAtSeconds != null) Command.Start(connectedAtSeconds) else Command.None
		else -> Command.Stop
	}
}
