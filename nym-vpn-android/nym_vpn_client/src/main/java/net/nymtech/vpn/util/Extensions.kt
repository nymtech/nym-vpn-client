package net.nymtech.vpn.util

import kotlinx.coroutines.channels.awaitClose
import kotlinx.coroutines.ensureActive
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.callbackFlow
import java.net.InetAddress
import kotlin.coroutines.coroutineContext

suspend inline fun <T> Flow<T>.safeCollect(crossinline action: suspend (T) -> Unit) {
    collect {
        coroutineContext.ensureActive()
        action(it)
    }
}

fun <T> EventNotifier<T>.callbackFlowFromSubscription(id: Any) = callbackFlow {
    this@callbackFlowFromSubscription.subscribe(id) {
        this.trySend(it)
    }
    awaitClose {
        this@callbackFlowFromSubscription.unsubscribe(id)
    }
}

fun InetAddress.addressString(): String {
    val hostNameAndAddress = this.toString().split('/', limit = 2)
    val address = hostNameAndAddress[1]

    return address
}