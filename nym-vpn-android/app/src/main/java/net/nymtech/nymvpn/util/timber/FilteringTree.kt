package net.nymtech.nymvpn.util.timber

import timber.log.Timber

class FilteringTree(
	private val minPriority: Int,
	private val delegate: Timber.Tree,
) : Timber.Tree() {

	override fun log(priority: Int, tag: String?, message: String, t: Throwable?) {
		if (priority < minPriority) return
		delegate.log(priority, tag, message, t)
	}
}
