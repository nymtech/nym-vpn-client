package net.nymtech.billing

import android.content.Context
import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers

fun initBilling(context: Context, applicationScope: CoroutineScope, ioDispatcher: CoroutineDispatcher = Dispatchers.IO): Billing = MockBilling()
