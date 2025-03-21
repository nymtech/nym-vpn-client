package net.nymtech.nymvpn.ui.screens.settings.logs.components

import androidx.compose.foundation.lazy.LazyListState
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.snapshotFlow

@Composable
fun AutoScrollEffect(
	logsSize: Int,
	lazyColumnListState: LazyListState,
	isAutoScrolling: Boolean,
	onAutoScrollingChange: (Boolean) -> Unit,
	lastScrollPosition: Int,
	onLastScrollPositionChange: (Int) -> Unit,
) {
	LaunchedEffect(isAutoScrolling) {
		if (isAutoScrolling) {
			lazyColumnListState.animateScrollToItem(logsSize)
		}
	}

	LaunchedEffect(lazyColumnListState) {
		snapshotFlow { lazyColumnListState.firstVisibleItemIndex }
			.collect { currentScrollPosition ->
				if (currentScrollPosition < lastScrollPosition && isAutoScrolling) {
					onAutoScrollingChange(false)
				}
				val visible = lazyColumnListState.layoutInfo.visibleItemsInfo
				if (visible.isNotEmpty()) {
					if (visible.last().index == lazyColumnListState.layoutInfo.totalItemsCount - 1 && !isAutoScrolling) {
						onAutoScrollingChange(true)
					}
				}
				onLastScrollPositionChange(currentScrollPosition)
			}
	}

	LaunchedEffect(logsSize) {
		if (isAutoScrolling) {
			lazyColumnListState.animateScrollToItem(logsSize)
		}
	}
}
