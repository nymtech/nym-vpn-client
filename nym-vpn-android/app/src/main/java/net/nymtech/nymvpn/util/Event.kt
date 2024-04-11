package net.nymtech.nymvpn.util

import net.nymtech.nymvpn.R

sealed class Event(val message: StringValue = StringValue.Empty) {
	sealed class Error(message: StringValue = StringValue.Empty) : Event(message) {
		data object None : Error()

		data object LoginFailed :
			Error(StringValue.StringResource(R.string.credential_failed_message))

		data class Exception(val exception: kotlin.Exception) :
			Error(
				exception.message?.let { StringValue.DynamicString(it) }
					?: StringValue.StringResource(R.string.unknown_error),
			)
	}

	sealed class Message(message: StringValue = StringValue.Empty) : Event(message) {
		data object None : Message()
	}
}
