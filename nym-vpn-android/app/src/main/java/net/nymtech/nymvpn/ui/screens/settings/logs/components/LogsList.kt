package net.nymtech.nymvpn.ui.screens.settings.logs.components

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.LazyListState
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalClipboardManager
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.unit.dp
import net.nymtech.logcatutil.model.LogMessage
import net.nymtech.nymvpn.util.extensions.scaledWidth

@Composable
fun LogsList(logs: List<LogMessage>, lazyColumnListState: LazyListState, modifier: Modifier = Modifier) {
	val clipboardManager = LocalClipboardManager.current

	LazyColumn(
		horizontalAlignment = Alignment.CenterHorizontally,
		verticalArrangement = Arrangement.spacedBy(16.dp, Alignment.Top),
		state = lazyColumnListState,
		modifier = modifier
			.fillMaxSize()
			.padding(top = 5.dp)
			.padding(horizontal = 24.dp.scaledWidth())
			.padding(bottom = 5.dp),
	) {
		itemsIndexed(
			items = logs,
			key = { index, _ -> index },
			contentType = { _, _ -> null },
		) { _, log ->
			LogRow(log = log, onClick = { clipboardManager.setText(AnnotatedString(log.toString())) })
		}
	}
}
