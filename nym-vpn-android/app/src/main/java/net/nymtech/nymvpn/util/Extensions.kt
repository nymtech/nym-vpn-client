package net.nymtech.nymvpn.util

import android.content.BroadcastReceiver
import android.content.Context
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.TextUnit
import androidx.navigation.NavController
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.DelicateCoroutinesApi
import kotlinx.coroutines.ExperimentalCoroutinesApi
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.ObsoleteCoroutinesApi
import kotlinx.coroutines.channels.ClosedReceiveChannelException
import kotlinx.coroutines.channels.ReceiveChannel
import kotlinx.coroutines.channels.produce
import kotlinx.coroutines.channels.ticker
import kotlinx.coroutines.coroutineScope
import kotlinx.coroutines.ensureActive
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.channelFlow
import kotlinx.coroutines.launch
import kotlinx.coroutines.selects.whileSelect
import net.nymtech.nymvpn.NymVpn
import timber.log.Timber
import java.time.Duration
import java.time.Instant
import java.util.concurrent.ConcurrentLinkedQueue
import kotlin.coroutines.CoroutineContext
import kotlin.coroutines.EmptyCoroutineContext
import kotlin.coroutines.cancellation.CancellationException
import kotlin.coroutines.coroutineContext

fun Dp.scaledHeight(): Dp {
	return NymVpn.resizeHeight(this)
}

fun Dp.scaledWidth(): Dp {
	return NymVpn.resizeWidth(this)
}

fun TextUnit.scaled(): TextUnit {
	return NymVpn.resizeHeight(this)
}

fun BroadcastReceiver.goAsync(context: CoroutineContext = EmptyCoroutineContext, block: suspend CoroutineScope.() -> Unit) {
	val pendingResult = goAsync()
	@OptIn(DelicateCoroutinesApi::class) // Must run globally; there's no teardown callback.
	GlobalScope.launch(context) {
		try {
			block()
		} finally {
			pendingResult.finish()
		}
	}
}

val Context.actionBarSize
	get() = theme.obtainStyledAttributes(intArrayOf(android.R.attr.actionBarSize))
		.let { attrs -> attrs.getDimension(0, 0F).toInt().also { attrs.recycle() } }

suspend inline fun <T> Flow<T>.safeCollect(crossinline action: suspend (T) -> Unit) {
	collect {
		coroutineContext.ensureActive()
		action(it)
	}
}

fun NavController.navigateNoBack(route: String) {
	navigate(route) {
		popUpTo(0)
	}
}

fun Instant.durationFromNow(): Duration {
	return Duration.between(Instant.now(), this)
}

/**
 * Chunks based on a time or size threshold.
 *
 * Borrowed from this [Stack Overflow question](https://stackoverflow.com/questions/51022533/kotlin-chunk-sequence-based-on-size-and-time).
 */
@OptIn(ObsoleteCoroutinesApi::class)
fun <T> ReceiveChannel<T>.chunked(scope: CoroutineScope, size: Int, time: Duration) = scope.produce<List<T>> {
	while (true) { // this loop goes over each chunk
		val chunk = ConcurrentLinkedQueue<T>() // current chunk
		val ticker = ticker(time.toMillis()) // time-limit for this chunk
		try {
			whileSelect {
				ticker.onReceive {
					false // done with chunk when timer ticks, takes priority over received elements
				}
				this@chunked.onReceive {
					chunk += it
					chunk.size < size // continue whileSelect if chunk is not full
				}
			}
		} catch (e: ClosedReceiveChannelException) {
			Timber.e(e)
			return@produce
		} finally {
			ticker.cancel()
			if (chunk.isNotEmpty()) {
				send(chunk.toList())
			}
		}
	}
}

@OptIn(DelicateCoroutinesApi::class)
fun <T> Flow<T>.chunked(size: Int, time: Duration) = channelFlow {
	coroutineScope {
		val channel = asChannel(this@chunked).chunked(this, size, time)
		try {
			while (!channel.isClosedForReceive) {
				send(channel.receive())
			}
		} catch (e: ClosedReceiveChannelException) {
			// Channel was closed by the flow completing, nothing to do
			Timber.w(e)
		} catch (e: CancellationException) {
			channel.cancel(e)
			throw e
		} catch (e: Exception) {
			channel.cancel(CancellationException("Closing channel due to flow exception", e))
			throw e
		}
	}
}

@ExperimentalCoroutinesApi
fun <T> CoroutineScope.asChannel(flow: Flow<T>): ReceiveChannel<T> = produce {
	flow.collect { value ->
		channel.send(value)
	}
}
