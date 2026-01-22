package net.nymtech.nymvpn.ui.screens.settings.logs.components

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyListState
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.unit.dp
import net.nymtech.logcatutil.model.LogMessage

@Composable
fun LogsList(logs: List<LogMessage>, lazyColumnListState: LazyListState, modifier: Modifier = Modifier) {
	LazyColumn(
		verticalArrangement = Arrangement.spacedBy(10.dp),
		state = lazyColumnListState,
		modifier = modifier
			.fillMaxSize()
			.padding(top = 8.dp),
	) {
		itemsIndexed(
			items = logs,
			key = { index, _ -> index },
		) { _, log ->
			LogsListItem(log = log)
		}
	}
}
