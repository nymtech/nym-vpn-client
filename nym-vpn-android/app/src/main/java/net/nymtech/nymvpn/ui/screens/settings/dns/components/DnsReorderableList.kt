package net.nymtech.nymvpn.ui.screens.settings.dns.components

import androidx.compose.foundation.gestures.detectDragGesturesAfterLongPress
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.DeleteOutline
import androidx.compose.material.icons.filled.DragIndicator
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.IconButton
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.unit.IntOffset
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import kotlin.math.roundToInt

@Composable
fun DnsReorderableList(dns: List<DnsEntry>, onMove: (from: Int, to: Int) -> Unit, onDelete: (index: Int) -> Unit) {
	val rowHeight = 56.dp
	val divider = 1.dp
	val totalHeight = (rowHeight + divider) * dns.size

	var draggingIndex by remember { mutableStateOf<Int?>(null) }
	var dragOffsetY by remember { mutableStateOf(0f) }

	LazyColumn(
		modifier = Modifier
			.fillMaxWidth()
			.height(totalHeight),
		userScrollEnabled = false,
	) {
		itemsIndexed(
			items = dns,
			key = { _, item -> item.id },
		) { index, item ->
			val isDragging = draggingIndex == index
			val offsetY = if (isDragging) dragOffsetY else 0f

			Row(
				modifier = Modifier
					.fillMaxWidth()
					.height(rowHeight)
					.offset { IntOffset(0, offsetY.roundToInt()) }
					.pointerInput(dns.size, item.id) {
						detectDragGesturesAfterLongPress(
							onDragStart = {
								draggingIndex = dns.indexOfFirst { it.id == item.id }.takeIf { it >= 0 }
								dragOffsetY = 0f
							},
							onDragEnd = {
								draggingIndex = null
								dragOffsetY = 0f
							},
							onDragCancel = {
								draggingIndex = null
								dragOffsetY = 0f
							},
							onDrag = { change, dragAmount ->
								change.consume()
								val from = draggingIndex ?: return@detectDragGesturesAfterLongPress

								dragOffsetY += dragAmount.y

								val threshold = rowHeight.toPx()
								val to = when {
									dragOffsetY > threshold && from < dns.lastIndex -> from + 1
									dragOffsetY < -threshold && from > 0 -> from - 1
									else -> from
								}

								if (to != from) {
									onMove(from, to)
									draggingIndex = to
									dragOffsetY = 0f
								}
							},
						)
					}
					.padding(vertical = 8.dp),
				verticalAlignment = Alignment.CenterVertically,
			) {
				Icon(
					imageVector = Icons.Default.DragIndicator,
					contentDescription = "Drag",
					tint = MaterialTheme.colorScheme.onBackground,
					modifier = Modifier.padding(end = 10.dp),
				)

				Text(
					text = item.value,
					style = MaterialTheme.typography.bodyLarge,
					fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
					modifier = Modifier.weight(1f),
				)

				IconButton(onClick = { onDelete(index) }) {
					Icon(
						imageVector = Icons.Default.DeleteOutline,
						contentDescription = stringResource(R.string.logs_delete),
					)
				}
			}
			HorizontalDivider(color = MaterialTheme.colorScheme.surfaceVariant)
		}
	}
}

fun <T> MutableList<T>.move(from: Int, to: Int) {
	if (from == to) return
	val item = removeAt(from)
	add(to, item)
}
