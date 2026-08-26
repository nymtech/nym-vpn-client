package net.nymtech.nymvpn.ui.screens.main.panel

import android.content.res.Configuration
import androidx.compose.foundation.background
import androidx.compose.foundation.gestures.detectVerticalDragGestures
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableFloatStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Modifier
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.model.ConnectionState
import net.nymtech.nymvpn.ui.screens.main.panel.components.ActionButton
import net.nymtech.nymvpn.ui.screens.main.panel.components.DragHandle
import net.nymtech.nymvpn.ui.screens.main.panel.components.ModeTabs
import net.nymtech.nymvpn.ui.screens.main.panel.components.NodeSection
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import nym_vpn_lib_types.AccountControllerState
import nym_vpn_lib_types.Score

@Composable
fun ConnectPanel(
	state: ConnectPanelState,
	onModeChange: (ConnectMode) -> Unit,
	onExitNodeClick: () -> Unit,
	onEntryNodeClick: () -> Unit,
	onExitInfoClick: () -> Unit,
	onEntryInfoClick: () -> Unit,
	onAction: (ConnectAction) -> Unit,
	onPanelStateChange: (PanelState) -> Unit,
	modifier: Modifier = Modifier,
) {
	var panelState by remember { mutableStateOf(state.initialPanelState) }
	val dragThresholdPx = with(LocalDensity.current) { 60.dp.toPx() }
	var dragAccum by remember { mutableFloatStateOf(0f) }

	fun changePanelState(new: PanelState) {
		panelState = new
		onPanelStateChange(new)
	}

	Column(
		modifier = modifier
			.fillMaxWidth()
			.pointerInput(Unit) {
				detectVerticalDragGestures(
					onDragStart = { dragAccum = 0f },
					onDragEnd = {
						changePanelState(
							when {
								dragAccum < -dragThresholdPx -> PanelState.FULL
								dragAccum > dragThresholdPx -> PanelState.COLLAPSED
								else -> panelState
							},
						)
						dragAccum = 0f
					},
					onDragCancel = { dragAccum = 0f },
					onVerticalDrag = { change, dragAmount ->
						change.consume()
						dragAccum += dragAmount
					},
				)
			},
	) {
		ModeTabs(
			selected = state.connectMode,
			onSelect = onModeChange,
			modifier = Modifier.padding(bottom = 14.dp),
		)

		Column(
			modifier = Modifier
				.fillMaxWidth()
				.background(
					color = MaterialTheme.colorScheme.surface,
					shape = RoundedCornerShape(16.dp),
				)
				.padding(16.dp),
		) {
			DragHandle(
				onClick = { changePanelState(if (panelState == PanelState.FULL) PanelState.COLLAPSED else PanelState.FULL) },
				modifier = Modifier.padding(bottom = 12.dp),
			)

			NodeSection(
				label = stringResource(R.string.one_click_nym_exit_node),
				node = state.exitNode,
				isClickable = true,
				onNodeClick = onExitNodeClick,
				onInfoClick = onExitInfoClick,
				visible = panelState == PanelState.FULL,
				alwaysShowRow = true,
			)

			NodeSection(
				label = stringResource(R.string.one_click_nym_entry_node),
				node = state.entryNode,
				isClickable = true,
				onNodeClick = onEntryNodeClick,
				onInfoClick = onEntryInfoClick,
				visible = panelState == PanelState.FULL,
				alwaysShowRow = false,
			)

			HorizontalDivider(
				thickness = 0.5.dp,
				color = MaterialTheme.colorScheme.surfaceBright,
			)

			Spacer(modifier.height(16.dp))

			ActionButton(
				connectionState = state.connectionState,
				accountState = state.accountState,
				isMnemonicStored = state.isMnemonicStored,
				isSubscriptionExpired = state.isSubscriptionExpired,
				hasSubscriptionHistory = state.hasSubscriptionHistory,
				onAction = onAction,
			)
		}
	}
}

@Preview(name = "Fast – light", uiMode = Configuration.UI_MODE_NIGHT_NO)
@Composable
private fun PreviewFastLight() {
	NymVPNTheme(Theme.LIGHT_MODE) {
		ConnectPanel(
			state = ConnectPanelState(
				connectionState = ConnectionState.Connected,
				accountState = AccountControllerState.ReadyToConnect,
				isMnemonicStored = true,
				connectMode = ConnectMode.FAST,
				exitNode = ServerNode(name = "Paris #1", countryCode = "fr", location = "France", score = Score.HIGH),
				entryNode = ServerNode(name = "Berlin #2", countryCode = "de", location = "Germany", score = Score.MEDIUM),
				initialPanelState = PanelState.FULL,
			),
			onModeChange = {},
			onAction = {},
			onPanelStateChange = {},
			onExitNodeClick = {},
			onEntryNodeClick = {},
			onExitInfoClick = {},
			onEntryInfoClick = {},
		)
	}
}
