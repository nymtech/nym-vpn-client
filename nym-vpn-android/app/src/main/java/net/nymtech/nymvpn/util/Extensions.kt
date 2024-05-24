package net.nymtech.nymvpn.util

import android.content.BroadcastReceiver
import android.content.Context
import androidx.compose.ui.unit.Dp
import androidx.compose.ui.unit.TextUnit
import androidx.navigation.NavController
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.DelicateCoroutinesApi
import kotlinx.coroutines.GlobalScope
import kotlinx.coroutines.ensureActive
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.NymVpn
import java.time.Duration
import java.time.Instant
import kotlin.coroutines.CoroutineContext
import kotlin.coroutines.EmptyCoroutineContext
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
