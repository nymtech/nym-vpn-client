package net.nymtech.nymvpn.service.gateway

import retrofit2.HttpException
import java.io.IOException

suspend fun <T> safeApiCall(apiCall: suspend () -> T): Result<T> {
	return try {
		Result.success(apiCall.invoke())
	} catch (throwable: Throwable) {
		when (throwable) {
			is IOException -> Result.failure(throwable)
			is HttpException -> Result.failure(throwable)
			else -> {
				Result.failure(throwable)
			}
		}
	}
}
