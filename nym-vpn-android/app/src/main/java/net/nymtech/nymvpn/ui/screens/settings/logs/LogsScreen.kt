package net.nymtech.nymvpn.ui.screens.settings.logs

import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.WindowInsets
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.lazy.LazyColumn
import androidx.compose.foundation.lazy.itemsIndexed
import androidx.compose.foundation.lazy.rememberLazyListState
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.automirrored.filled.ArrowBack
import androidx.compose.material.icons.outlined.Delete
import androidx.compose.material.icons.outlined.Share
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.NavigationBar
import androidx.compose.material3.NavigationBarItem
import androidx.compose.material3.NavigationBarItemDefaults
import androidx.compose.material3.Scaffold
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.rememberCoroutineScope
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.ClipboardManager
import androidx.compose.ui.platform.LocalClipboardManager
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import kotlinx.coroutines.launch
import net.nymtech.logcatutil.model.LogMessage
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.AppViewModel
import net.nymtech.nymvpn.ui.common.Modal
import net.nymtech.nymvpn.ui.common.buttons.MainStyledButton
import net.nymtech.nymvpn.ui.common.labels.LogTypeLabel
import net.nymtech.nymvpn.ui.common.navigation.NavBarState
import net.nymtech.nymvpn.ui.common.navigation.NavIcon
import net.nymtech.nymvpn.ui.common.navigation.NavTitle
import net.nymtech.nymvpn.ui.theme.CustomTypography
import net.nymtech.nymvpn.util.extensions.scaledWidth

@Composable
fun LogsScreen(appViewModel: AppViewModel, viewModel: LogsViewModel = hiltViewModel()) {
	val lazyColumnListState = rememberLazyListState()
	val clipboardManager: ClipboardManager = LocalClipboardManager.current
	val scope = rememberCoroutineScope()
	var showModal by remember { mutableStateOf(false) }

	val context = LocalContext.current

	val logs = viewModel.logs

	LaunchedEffect(Unit) {
		appViewModel.onNavBarStateChange(
			NavBarState(
				title = { NavTitle(stringResource(R.string.logs)) },
				leading = {
					NavIcon(Icons.AutoMirrored.Filled.ArrowBack) {
						appViewModel.navController.popBackStack()
					}
				},
			),
		)
	}

	LaunchedEffect(logs.size) {
		scope.launch {
			lazyColumnListState.animateScrollToItem(logs.size)
		}
	}

	Modal(showModal, { showModal = false }, { Text(stringResource(R.string.delete_logs_title), style = CustomTypography.labelHuge) }, {
		Text(stringResource(R.string.delete_logs_description), textAlign = TextAlign.Center, style = MaterialTheme.typography.bodyMedium)
	}, icon = Icons.Outlined.Delete, confirmButton = {
		MainStyledButton(
			onClick = {
				viewModel.deleteLogs()
				showModal = false
			},
			content = {
				Text(text = stringResource(id = R.string.yes))
			},
		)
	})

	Scaffold(
		contentWindowInsets = WindowInsets(0.dp),
		bottomBar = {
			NavigationBar(
				containerColor = MaterialTheme.colorScheme.surface,
				tonalElevation = 0.dp,
			) {
				listOf(
					NavigationBarItem(
						colors = NavigationBarItemDefaults.colors().copy(
							unselectedIconColor = MaterialTheme.colorScheme.onSurface,
							unselectedTextColor = MaterialTheme.colorScheme.onSurface,
						),
						selected = false,
						onClick = {
							viewModel.shareLogs(context)
						},
						label = {
							Text(
								text = stringResource(R.string.share),
								style = MaterialTheme.typography.labelMedium,
							)
						},
						icon = {
							val icon = Icons.Outlined.Share
							Icon(
								imageVector = icon,
								contentDescription = icon.name,
							)
						},
					),
					NavigationBarItem(
						colors = NavigationBarItemDefaults.colors().copy(
							unselectedIconColor = MaterialTheme.colorScheme.onSurface,
							unselectedTextColor = MaterialTheme.colorScheme.onSurface,
						),
						selected = false,
						onClick = {
							showModal = true
						},
						label = {
							Text(
								text = stringResource(R.string.delete),
								style = MaterialTheme.typography.labelMedium,
							)
						},
						icon = {
							val icon = Icons.Outlined.Delete
							Icon(
								imageVector = icon,
								contentDescription = icon.name,
							)
						},
					),
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
				.padding(horizontal = 24.dp.scaledWidth()).padding(it).padding(bottom = 5.dp),
		) {
			itemsIndexed(logs, key = { index, _ -> index }, contentType = { _: Int, _: LogMessage -> null }) { _, it ->
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
