package net.nymtech.nymvpn.util

import android.content.Context
import net.nymtech.nymvpn.R

sealed class NymVpnExceptions : Exception() {
	abstract fun getMessage(context: Context): String
	data class MissingCredentialException(
		private val userMessage: StringValue = StringValue.StringResource(R.string.exception_cred_missing),
	) : NymVpnExceptions() {
		override fun getMessage(context: Context): String {
			return userMessage.asString(context)
		}
	}

	data class InvalidCredentialException(
		private val userMessage: StringValue = StringValue.StringResource(
			R.string.exception_cred_invalid,
		),
	) : NymVpnExceptions() {
		override fun getMessage(context: Context): String {
			return userMessage.asString(context)
		}
	}

	data class PermissionsNotGrantedException(
		private val userMessage: StringValue = StringValue.StringResource(
			R.string.exception_permission_not_granted,
		),
	) : NymVpnExceptions() {
		override fun getMessage(context: Context): String {
			return userMessage.asString(context)
		}
	}
}
