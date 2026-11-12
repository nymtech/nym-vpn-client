package net.nymtech.nymvpn.manager.freepass

import net.nymtech.vpn.model.freepass.FreepassResult

interface FreepassManager {
	// Creates an account with the pass when none is stored, else applies it.
	suspend fun redeem(code: String): FreepassResult
}
