package net.nymtech.nymvpn.util

import net.nymtech.nymvpn.NymVPN
import timber.log.Timber

sealed class Result<T> {
    class Success<T>(val data: T) : Result<T>()

    class Error<T>(val error: Event.Error) : Result<T>() {
        init {
            when (this.error) {
                is Event.Error.Exception -> Timber.e(this.error.exception)
                else -> Timber.e(this.error.message.asString(NymVPN.instance))
            }
        }
    }
}