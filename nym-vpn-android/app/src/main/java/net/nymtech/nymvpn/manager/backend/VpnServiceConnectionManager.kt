package net.nymtech.nymvpn.manager.backend

import android.content.ComponentName
import android.content.Context
import android.content.Intent
import android.content.ServiceConnection
import android.os.IBinder
import dagger.hilt.android.qualifiers.ApplicationContext
import kotlinx.coroutines.CompletableDeferred
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.SupervisorJob
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.StateFlow
import kotlinx.coroutines.flow.asStateFlow
import kotlinx.coroutines.launch
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import kotlinx.coroutines.withContext
import kotlinx.coroutines.withTimeout
import net.nymtech.nymvpn.di.qualifiers.IoDispatcher
import net.nymtech.vpn.backend.api.VpnServiceApi
import net.nymtech.vpn.backend.service.VpnService
import timber.log.Timber
import javax.inject.Inject
import javax.inject.Singleton

/**
 * Binds directly to VpnService and exposes a cached VpnServiceApi instance.
 * No VpnApiService proxy anymore.
 */
@Singleton
class VpnServiceConnectionManager @Inject constructor(@ApplicationContext private val context: Context, @IoDispatcher private val ioDispatcher: CoroutineDispatcher) {

	companion object {
		private const val TAG = "vpn-service-conn"
		private const val DEFAULT_TIMEOUT_MS = 10_000L
	}

	private val scope = CoroutineScope(SupervisorJob() + ioDispatcher)

	private val _apiFlow = MutableStateFlow<VpnServiceApi?>(null)
	val apiFlow: StateFlow<VpnServiceApi?> = _apiFlow.asStateFlow()

	@Volatile
	private var bindInProgress: CompletableDeferred<VpnServiceApi>? = null

	private val bindMutex = Mutex()

	private val serviceConnection = object : ServiceConnection {

		override fun onServiceConnected(name: ComponentName?, service: IBinder?) {
			Timber.tag(TAG).i("onServiceConnected name=%s binder=%s", name, service != null)

			val binder = service as? VpnService.LocalBinder
			if (binder == null) {
				failBind(IllegalStateException("Binder is not VpnService.LocalBinder"))
				return
			}

			val api = binder.api()
			_apiFlow.value = api
			bindInProgress?.complete(api)
			bindInProgress = null
		}

		override fun onServiceDisconnected(name: ComponentName?) {
			Timber.tag(TAG).w("onServiceDisconnected name=%s", name)
			_apiFlow.value = null
		}

		override fun onNullBinding(name: ComponentName?) {
			Timber.tag(TAG).e("onNullBinding name=%s", name)
			failBind(IllegalStateException("Received onNullBinding"))
		}
	}

	suspend fun awaitApi(timeoutMs: Long = DEFAULT_TIMEOUT_MS): VpnServiceApi {
		apiFlow.value?.let { return it }

		val deferred: CompletableDeferred<VpnServiceApi> = bindMutex.withLock {
			apiFlow.value?.let { existingApi ->
				return@withLock CompletableDeferred<VpnServiceApi>().apply { complete(existingApi) }
			}

			bindInProgress?.let { existingDeferred ->
				return@withLock existingDeferred
			}

			CompletableDeferred<VpnServiceApi>().also { newDeferred ->
				bindInProgress = newDeferred
				scope.launch {
					runCatching { bindServiceInternal() }
						.onFailure { t -> failBind(t) }
				}
			}
		}

		return withTimeout(timeoutMs) { deferred.await() }
	}

	suspend fun <T> withApi(timeoutMs: Long = DEFAULT_TIMEOUT_MS, block: suspend (VpnServiceApi) -> T): T {
		val api = awaitApi(timeoutMs)
		return withContext(ioDispatcher) { block(api) }
	}

	private fun bindServiceInternal() {
		val intent = Intent(context, VpnService::class.java).apply {
			action = VpnServiceApi.ACTION_BIND_APP
		}
		val ok = context.bindService(intent, serviceConnection, Context.BIND_AUTO_CREATE)
		if (!ok) throw IllegalStateException("bindService returned false")

		Timber.tag(TAG).i("bindService requested service=VpnService ok=true")
	}

	private fun failBind(t: Throwable) {
		Timber.tag(TAG).e(t, "BindFailed")

		val deferred = bindInProgress
		bindInProgress = null
		_apiFlow.value = null

		runCatching { deferred?.completeExceptionally(t) }
	}
}
