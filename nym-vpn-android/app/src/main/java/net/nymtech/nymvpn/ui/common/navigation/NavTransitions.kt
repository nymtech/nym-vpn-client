package net.nymtech.nymvpn.ui.common.navigation

import androidx.compose.animation.AnimatedContentTransitionScope
import androidx.compose.animation.AnimatedContentTransitionScope.SlideDirection
import androidx.compose.animation.EnterTransition
import androidx.compose.animation.ExitTransition
import androidx.compose.animation.core.CubicBezierEasing
import androidx.compose.animation.core.tween
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.ui.unit.IntOffset
import androidx.navigation.NavBackStackEntry

typealias NavTransitionScope = AnimatedContentTransitionScope<NavBackStackEntry>

// Pop and predictive pop must share a spec, otherwise the screen jumps on gesture commit.
object NavTransitions {
	private const val SHARED_AXIS_DURATION_MS = 300
	private const val FADE_DURATION_MS = 200

	private val emphasizedDecelerate = CubicBezierEasing(0.05f, 0.7f, 0.1f, 1f)
	private val slideSpec = tween<IntOffset>(SHARED_AXIS_DURATION_MS, easing = emphasizedDecelerate)
	private val sharedAxisFadeSpec = tween<Float>(SHARED_AXIS_DURATION_MS, easing = emphasizedDecelerate)
	private val fadeSpec = tween<Float>(FADE_DURATION_MS)

	private val slideOffset: (Int) -> Int = { fullWidth -> fullWidth / 10 }

	fun NavTransitionScope.enter(fade: Boolean): EnterTransition = if (fade) fadeIn(fadeSpec) else sharedAxisEnter(SlideDirection.Start)

	fun NavTransitionScope.exit(fade: Boolean): ExitTransition = if (fade) fadeOut(fadeSpec) else sharedAxisExit(SlideDirection.Start)

	fun NavTransitionScope.popEnter(fade: Boolean): EnterTransition = if (fade) fadeIn(fadeSpec) else sharedAxisEnter(SlideDirection.End)

	fun NavTransitionScope.popExit(fade: Boolean): ExitTransition = if (fade) fadeOut(fadeSpec) else sharedAxisExit(SlideDirection.End)

	private fun NavTransitionScope.sharedAxisEnter(towards: SlideDirection): EnterTransition = slideIntoContainer(towards, slideSpec, slideOffset) + fadeIn(sharedAxisFadeSpec)

	private fun NavTransitionScope.sharedAxisExit(towards: SlideDirection): ExitTransition = slideOutOfContainer(towards, slideSpec, slideOffset) + fadeOut(sharedAxisFadeSpec)
}
