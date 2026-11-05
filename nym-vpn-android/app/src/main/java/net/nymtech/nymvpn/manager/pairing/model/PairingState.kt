package net.nymtech.nymvpn.manager.pairing.model

import net.nymtech.vpn.model.pairing.PairingFailure
import kotlin.time.TimeMark

// URLs and PINs are kept out of toString so states can be logged.
sealed interface PairingState {
	data object Idle : PairingState

	data object StartingCode : PairingState

	// expiresAt is a client-side estimate; the core's TimedOut event is authoritative.
	data class ShowingCode(val url: String, val sessionId: UInt, val expiresAt: TimeMark) : PairingState {
		override fun toString(): String = "ShowingCode(sessionId=$sessionId)"
	}

	data class ConfirmCode(val pin: String, val sessionId: UInt) : PairingState {
		override fun toString(): String = "ConfirmCode(sessionId=$sessionId)"
	}

	data object SharingAccount : PairingState

	data object AccountShared : PairingState

	data class WaitingForApproval(val pin: String, val sessionId: UInt) : PairingState {
		override fun toString(): String = "WaitingForApproval(sessionId=$sessionId)"
	}

	data object SignedIn : PairingState

	data object Expired : PairingState

	data class Failed(val failure: PairingFailure) : PairingState
}

internal val PairingState.activeSessionId: UInt?
	get() = when (this) {
		is PairingState.ShowingCode -> sessionId
		is PairingState.ConfirmCode -> sessionId
		is PairingState.WaitingForApproval -> sessionId
		else -> null
	}
