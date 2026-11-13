package net.nymtech.nymvpn.manager.secureenclave

import nym_secure_enclave.SecureEnclaveException
import javax.inject.Inject
import javax.inject.Singleton

/**
 * Tracks whether the user unlocked the secure enclave. Starts locked.
 *
 * The keystore cannot revoke an authentication window, so locking is enforced here: while locked,
 * keys requiring user presence fail with [SecureEnclaveException.Locked].
 */
@Singleton
class SecureEnclaveSession @Inject constructor() {

	private val lock = Any()
	private val unlockLock = Any()
	private var lockEpoch = 0L

	@Volatile
	var isUnlocked = false
		private set

	/**
	 * Runs [authenticate] and marks the session unlocked, unless [lock] was called in the meantime.
	 * Does nothing when the session is unlocked and [isAuthenticated] reports that the keystore
	 * authentication window is still open. Concurrent calls are serialized, so the user is prompted
	 * at most once.
	 */
	fun unlock(isAuthenticated: () -> Boolean, authenticate: () -> Unit) {
		synchronized(unlockLock) {
			if (!isUnlocked || !isAuthenticated()) {
				val epoch = synchronized(lock) { lockEpoch }
				authenticate()
				synchronized(lock) {
					if (epoch != lockEpoch) throw SecureEnclaveException.UserCancelled()
					isUnlocked = true
				}
			}
		}
	}

	fun lock() {
		synchronized(lock) {
			lockEpoch++
			isUnlocked = false
		}
	}
}
