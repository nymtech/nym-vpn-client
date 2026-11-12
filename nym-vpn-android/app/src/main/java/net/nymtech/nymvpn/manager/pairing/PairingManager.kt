package net.nymtech.nymvpn.manager.pairing

import kotlinx.coroutines.flow.StateFlow
import net.nymtech.nymvpn.manager.pairing.model.PairingState
import net.nymtech.vpn.model.pairing.PairingCommandResult

interface PairingManager {
	val state: StateFlow<PairingState>

	// Callers verify device auth before showing a code.
	suspend fun startShowingCode()

	suspend fun confirmCode()

	suspend fun onScannedKey(publicKey: String): PairingCommandResult<Unit>

	suspend fun cancel()
}
