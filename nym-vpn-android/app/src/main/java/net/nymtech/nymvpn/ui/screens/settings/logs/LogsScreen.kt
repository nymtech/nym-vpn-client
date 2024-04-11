package net.nymtech.nymvpn.ui.screens.settings.logs

import android.annotation.SuppressLint
import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.items
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.filled.Save
import androidx.compose.material3.FloatingActionButton
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.ClipboardManager
import androidx.compose.ui.platform.LocalClipboardManager
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.launch
import net.nymtech.nymvpn.ui.AppViewModel
import net.nymtech.nymvpn.ui.common.labels.LogTypeLabel
import net.nymtech.nymvpn.util.scaledWidth

@SuppressLint("UnusedMaterial3ScaffoldPaddingParameter")
@Composable
fun LogsScreen(appViewModel: AppViewModel) {
	val logs =
		remember {
			appViewModel.logs
		}

	val lazyColumnListState = rememberLazyListState()
	val clipboardManager: ClipboardManager = LocalClipboardManager.current
	val scope = rememberCoroutineScope()

	LaunchedEffect(logs.size) {
		scope.launch {
			lazyColumnListState.animateScrollToItem(logs.size)
		}
	}

	Scaffold(
		floatingActionButton = {
			FloatingActionButton(
				onClick = {
					appViewModel.saveLogsToFile()
				},
				shape = RoundedCornerShape(16.dp),
				containerColor = MaterialTheme.colorScheme.primary,
			) {
				val icon = Icons.Filled.Save
				Icon(
					imageVector = icon,
					contentDescription = icon.name,
					tint = MaterialTheme.colorScheme.onPrimary,
				)
			}
		},
	) {
		LazyColumn(
			horizontalAlignment = Alignment.CenterHorizontally,
			verticalArrangement = Arrangement.spacedBy(16.dp, Alignment.Top),
			state = lazyColumnListState,
			modifier =
			Modifier
				.fillMaxSize()
				.padding(top = 5.dp)
				.padding(horizontal = 24.dp.scaledWidth()),
		) {
			items(logs) {
				Row(
					horizontalArrangement = Arrangement.spacedBy(5.dp, Alignment.Start),
					verticalAlignment = Alignment.Top,
					modifier =
					Modifier
						.fillMaxSize()
						.clickable(
							interactionSource = remember { MutableInteractionSource() },
							indication = null,
							onClick = {
								clipboardManager.setText(
									annotatedString = AnnotatedString(it.toString()),
								)
							},
						),
				) {
					Text(
						text = it.tag,
						modifier = Modifier.fillMaxSize(0.3f),
						style = MaterialTheme.typography.labelSmall,
					)
					LogTypeLabel(color = Color(it.level.color())) {
						Text(
							text = it.level.signifier,
							textAlign = TextAlign.Center,
							style = MaterialTheme.typography.labelSmall,
						)
					}
					Text(
						"${it.message} - ${it.time}",
						color = MaterialTheme.colorScheme.onBackground,
						style = MaterialTheme.typography.labelSmall,
					)
				}
			}
		}
	}
}
