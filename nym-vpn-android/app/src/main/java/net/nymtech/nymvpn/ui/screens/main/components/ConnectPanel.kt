package net.nymtech.nymvpn.ui.screens.main.components

import android.content.res.Configuration
import androidx.compose.animation.AnimatedVisibility
import androidx.compose.animation.core.animateDpAsState
import androidx.compose.animation.core.tween
import androidx.compose.animation.expandVertically
import androidx.compose.animation.fadeIn
import androidx.compose.animation.fadeOut
import androidx.compose.animation.shrinkVertically
import androidx.compose.foundation.BorderStroke
import androidx.compose.foundation.Image
import androidx.compose.foundation.background
import androidx.compose.foundation.clickable
import androidx.compose.foundation.gestures.detectVerticalDragGestures
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.offset
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.CircleShape
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Bolt
import androidx.compose.material.icons.outlined.KeyboardArrowDown
import androidx.compose.material.icons.outlined.KeyboardArrowUp
import androidx.compose.material.icons.outlined.SignalCellularAlt
import androidx.compose.material3.HorizontalDivider
import androidx.compose.material3.Icon
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.material3.ripple
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableFloatStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.draw.clip
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.input.pointer.pointerInput
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.platform.LocalDensity
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.res.vectorResource
import androidx.compose.ui.text.font.FontWeight
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.text.style.TextOverflow
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.common.buttons.MainStyledButton
import net.nymtech.nymvpn.ui.model.ConnectionState
import net.nymtech.nymvpn.ui.theme.CustomTypography
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.ui.theme.iconSize
import net.nymtech.nymvpn.util.extensions.getFlagImageVectorByName
import net.nymtech.nymvpn.util.extensions.isVpnAlwaysOn
import net.nymtech.nymvpn.util.extensions.scaledHeight
import net.nymtech.vpn.backend.Tunnel
import nym_vpn_lib_types.AccountControllerState
import nym_vpn_lib_types.ErrorStateReason

enum class PanelState { COLLAPSED, MODE, FULL }

data class ServerNode(val name: String?, val countryCode: String?, val location: String?, val showQuicIcon: Boolean = false, val showQuicLewesIcon: Boolean = false, val isRandom: Boolean = false)

@Composable
fun ConnectPanel(
	connectionState: ConnectionState,
	accountState: AccountControllerState,
	isMnemonicStored: Boolean,
	vpnMode: Tunnel.Mode,
	exitNode: ServerNode,
	entryNode: ServerNode,
	onExitNodeClick: () -> Unit,
	onEntryNodeClick: () -> Unit,
	initialPanelState: PanelState,
	onFastModeClick: () -> Unit,
	onAnonModeClick: () -> Unit,
	onConnect: () -> Unit,
	onDisconnect: () -> Unit,
	onStopKillSwitch: () -> Unit,
	onGetStartedClick: () -> Unit,
	onPanelStateChange: (state: PanelState) -> Unit,
	modifier: Modifier = Modifier,
) {
	val context = LocalContext.current
	var panelState by remember { mutableStateOf(initialPanelState) }
	val dragThresholdPx = with(LocalDensity.current) { 60.dp.toPx() }
	var dragAccum by remember { mutableFloatStateOf(0f) }

	val canToggle = connectionState !is ConnectionState.Connecting &&
		connectionState != ConnectionState.Connected

	fun changePanelState(new: PanelState) {
		panelState = new
		onPanelStateChange(new)
	}

	Column(
		modifier = modifier
			.fillMaxWidth()
			.pointerInput(canToggle) {
				detectVerticalDragGestures(
					onDragStart = { dragAccum = 0f },
					onDragEnd = {
						if (canToggle) {
							changePanelState(
								when {
									dragAccum < -dragThresholdPx -> when (panelState) {
										PanelState.COLLAPSED -> PanelState.MODE
										PanelState.MODE -> PanelState.FULL
										PanelState.FULL -> PanelState.FULL
									}
									dragAccum > dragThresholdPx -> when (panelState) {
										PanelState.COLLAPSED -> PanelState.COLLAPSED
										PanelState.MODE -> PanelState.COLLAPSED
										PanelState.FULL -> PanelState.MODE
									}
									else -> panelState
								},
							)
						}
						dragAccum = 0f
					},
					onDragCancel = { dragAccum = 0f },
					onVerticalDrag = { change, dragAmount ->
						change.consume()
						dragAccum += dragAmount
					},
				)
			}
			.padding(horizontal = 15.dp)
			.padding(bottom = 20.dp, top = 8.dp),
	) {
		Box(
			modifier = Modifier
				.fillMaxWidth()
				.padding(bottom = 12.dp),
			contentAlignment = Alignment.Center,
		) {
			Box(
				modifier = Modifier
					.size(width = 32.dp, height = 4.dp)
					.clip(RoundedCornerShape(2.dp))
					.background(MaterialTheme.colorScheme.onSurfaceVariant.copy(alpha = 0.4f)),
			)
		}

		AnimatedVisibility(
			visible = panelState != PanelState.COLLAPSED,
			enter = expandVertically(animationSpec = tween(350)) + fadeIn(animationSpec = tween(350)),
			exit = shrinkVertically(animationSpec = tween(350)) + fadeOut(animationSpec = tween(350)),
		) {
			Column(
				verticalArrangement = Arrangement.spacedBy(16.dp),
				modifier = Modifier.padding(bottom = 16.dp),
			) {
				ModeToggle(
					vpnMode = vpnMode,
					onFastClick = onFastModeClick,
					onAnonClick = onAnonModeClick,
				)
				HorizontalDivider(
					thickness = 0.5.dp,
					color = MaterialTheme.colorScheme.surfaceBright,
				)
			}
		}

		AnimatedVisibility(
			visible = panelState == PanelState.FULL,
			enter = expandVertically(animationSpec = tween(350)) + fadeIn(animationSpec = tween(350)),
			exit = shrinkVertically(animationSpec = tween(350)) + fadeOut(animationSpec = tween(350)),
		) {
			Text(
				text = stringResource(R.string.one_click_nym_exit_node),
				style = MaterialTheme.typography.labelSmall,
				color = MaterialTheme.colorScheme.onSurfaceVariant,
				modifier = Modifier.padding(bottom = 4.dp),
			)
		}

		ServerRow(
			node = exitNode,
			onExpand = if (canToggle) {
				(
					{
						changePanelState(if (panelState == PanelState.COLLAPSED) PanelState.MODE else PanelState.COLLAPSED)
					}
					)
			} else {
				null
			},
			onCollapse = if (canToggle && panelState != PanelState.COLLAPSED) {
				(
					{
						changePanelState(if (panelState == PanelState.FULL) PanelState.MODE else PanelState.FULL)
					}
					)
			} else {
				null
			},
			modifier = Modifier.padding(bottom = 16.dp),
			currentState = panelState,
			connectionState = connectionState,
			onServerClick = onExitNodeClick,
		)

		AnimatedVisibility(
			visible = panelState == PanelState.FULL,
			enter = expandVertically(animationSpec = tween(350)) + fadeIn(animationSpec = tween(350)),
			exit = shrinkVertically(animationSpec = tween(350)) + fadeOut(animationSpec = tween(350)),
		) {
			Column {
				Text(
					text = stringResource(R.string.one_click_nym_entry_node),
					style = MaterialTheme.typography.labelSmall,
					color = MaterialTheme.colorScheme.onSurfaceVariant,
					modifier = Modifier.padding(bottom = 4.dp),
				)
				ServerRow(
					node = entryNode,
					fillTrailingSpace = true,
					modifier = Modifier.padding(bottom = 16.dp),
					currentState = panelState,
					connectionState = connectionState,
					onServerClick = onEntryNodeClick,
				)
			}
		}

		ActionButton(
			connectionState = connectionState,
			accountState = accountState,
			isMnemonicStored = isMnemonicStored,
			onConnect = onConnect,
			onDisconnect = onDisconnect,
			onStopKillSwitch = onStopKillSwitch,
			onGetStartedClick = onGetStartedClick,
		)
	}
}

@Composable
private fun ModeToggle(vpnMode: Tunnel.Mode, onFastClick: () -> Unit, onAnonClick: () -> Unit, modifier: Modifier = Modifier) {
	val isFast = vpnMode == Tunnel.Mode.TWO_HOP_MIXNET
	val indicatorX by animateDpAsState(
		targetValue = if (isFast) 6.dp else 46.dp,
		label = "toggle_indicator",
	)

	Row(
		verticalAlignment = Alignment.CenterVertically,
		horizontalArrangement = Arrangement.spacedBy(16.dp),
		modifier = modifier.fillMaxWidth(),
	) {
		Text(
			text = stringResource(R.string.one_click_mode_fast),
			style = MaterialTheme.typography.labelMedium.copy(
				fontWeight = if (isFast) FontWeight.Bold else FontWeight.Normal,
			),
			color = if (isFast) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.onSurfaceVariant,
			textAlign = TextAlign.End,
			modifier = Modifier
				.weight(1f)
				.clickable(
					onClick = onFastClick,
					interactionSource = remember { MutableInteractionSource() },
					indication = null,
				),
		)

		Box(
			modifier = Modifier
				.size(width = 80.dp, height = 40.dp)
				.clip(RoundedCornerShape(50))
				.background(Color(0xFF090909))
				.clickable { if (isFast) onAnonClick() else onFastClick() },
		) {
			Box(
				modifier = Modifier
					.offset(x = indicatorX, y = 6.dp)
					.size(28.dp)
					.clip(CircleShape)
					.background(Color.White),
				contentAlignment = Alignment.Center,
			) {
				Icon(
					imageVector = Icons.Outlined.Bolt,
					contentDescription = null,
					tint = Color.Black,
					modifier = Modifier.size(20.dp),
				)
			}
		}

		Text(
			text = stringResource(R.string.anonymous),
			style = MaterialTheme.typography.labelMedium.copy(
				fontWeight = if (!isFast) FontWeight.Bold else FontWeight.Normal,
			),
			color = if (!isFast) MaterialTheme.colorScheme.primary else MaterialTheme.colorScheme.onSurfaceVariant,
			modifier = Modifier
				.weight(1f)
				.clickable(
					interactionSource = remember { MutableInteractionSource() },
					indication = null,
					onClick = onAnonClick,
				),
		)
	}
}

@Composable
private fun ServerRow(
	node: ServerNode,
	modifier: Modifier = Modifier,
	onExpand: (() -> Unit)? = null,
	onCollapse: (() -> Unit)? = null,
	fillTrailingSpace: Boolean = false,
	onServerClick: () -> Unit,
	currentState: PanelState,
	connectionState: ConnectionState,
) {
	val context = LocalContext.current
	val isAutoMode = currentState == PanelState.COLLAPSED && (connectionState !is ConnectionState.Connecting && connectionState !is ConnectionState.Connected)
	val indication = if (!isAutoMode) ripple() else null

	Column(modifier = modifier.fillMaxWidth()) {
		Row(
			verticalAlignment = Alignment.CenterVertically,
			horizontalArrangement = Arrangement.spacedBy(8.dp),
		) {
			Icon(
				imageVector = Icons.Outlined.SignalCellularAlt,
				contentDescription = null,
				tint = MaterialTheme.colorScheme.primary,
				modifier = Modifier.size(iconSize),
			)

			Column(
				verticalArrangement = Arrangement.spacedBy(2.dp),
				modifier = Modifier.weight(1f)
					.clickable(
						interactionSource = remember { MutableInteractionSource() },
						indication = indication,
					) {
						if (!isAutoMode) {
							onServerClick()
						}
					},
			) {
				Row(
					verticalAlignment = Alignment.CenterVertically,
					horizontalArrangement = Arrangement.spacedBy(4.dp),
				) {
					if (!isAutoMode) {
						val (image, description) = node.countryCode?.let {
							Pair(
								ImageVector.vectorResource(context.getFlagImageVectorByName(it)),
								stringResource(R.string.country_flag, it),
							)
						} ?: Pair(
							ImageVector.vectorResource(R.drawable.faq),
							stringResource(R.string.unknown),
						)

						Image(
							image,
							description,
							modifier = Modifier
								.size(16.dp),
						)
					}

					val title = if (isAutoMode) {
						stringResource(R.string.one_click_auto_server)
					} else {
						node.name ?: stringResource(R.string.one_click_auto_server)
					}

					Text(
						text = title,
						style = MaterialTheme.typography.bodyMedium,
						color = MaterialTheme.colorScheme.onSurface,
						maxLines = 1,
						overflow = TextOverflow.Ellipsis,
						modifier = Modifier.weight(1f),
					)
				}

				AnimatedVisibility(
					visible = !isAutoMode && node.location != null && !node.isRandom,
					enter = expandVertically(animationSpec = tween(350)) + fadeIn(animationSpec = tween(350)),
					exit = shrinkVertically(animationSpec = tween(350)) + fadeOut(animationSpec = tween(350)),
				) {
					Text(
						text = node.location.orEmpty(),
						style = MaterialTheme.typography.labelSmall,
						color = MaterialTheme.colorScheme.onSurfaceVariant,
						maxLines = 1,
						overflow = TextOverflow.Ellipsis,
					)
				}
			}

			if (node.showQuicIcon) {
				Icon(
					imageVector = ImageVector.vectorResource(R.drawable.quic_label),
					contentDescription = null,
					tint = MaterialTheme.colorScheme.primary,
					modifier = Modifier.size(iconSize),
				)
			}

			if (!isAutoMode) {
				Icon(
					imageVector = ImageVector.vectorResource(R.drawable.ic_lewes),
					contentDescription = null,
					tint = MaterialTheme.colorScheme.primary,
					modifier = Modifier.size(iconSize),
				)
			}

			if (onExpand != null || onCollapse != null) {
				Column(horizontalAlignment = Alignment.CenterHorizontally) {
					if (onExpand != null) {
						Icon(
							imageVector = Icons.Outlined.KeyboardArrowUp,
							contentDescription = null,
							tint = MaterialTheme.colorScheme.onSurfaceVariant,
							modifier = Modifier
								.size(24.dp)
								.clickable { onExpand() },
						)
					}
					if (onCollapse != null) {
						Icon(
							imageVector = Icons.Outlined.KeyboardArrowDown,
							contentDescription = null,
							tint = MaterialTheme.colorScheme.onSurfaceVariant,
							modifier = Modifier
								.size(24.dp)
								.clickable { onCollapse() },
						)
					}
				}
			} else if (fillTrailingSpace) {
				Spacer(modifier = Modifier.size(24.dp))
			}
		}
	}
}

@Composable
private fun ActionButton(
	connectionState: ConnectionState,
	accountState: AccountControllerState,
	isMnemonicStored: Boolean,
	onConnect: () -> Unit,
	onDisconnect: () -> Unit,
	onStopKillSwitch: () -> Unit,
	onGetStartedClick: () -> Unit,
	modifier: Modifier = Modifier,
) {
	val context = LocalContext.current
	val buttonModifier = modifier
		.fillMaxWidth()
		.height(48.dp.scaledHeight())

	when (connectionState) {
		ConnectionState.Disconnected,
		ConnectionState.Offline,
		ConnectionState.WaitingForConnection,
		-> MainStyledButton(
			onClick = if (isMnemonicStored) onConnect else onGetStartedClick,
			content = {
				Text(
					stringResource(if (isMnemonicStored) R.string.connect else R.string.get_started),
					style = CustomTypography.buttonMain,
				)
			},
			modifier = buttonModifier,
			shape = RoundedCornerShape(50),
		)

		is ConnectionState.Connecting -> MainStyledButton(
			onClick = onDisconnect,
			content = {
				Text(
					stringResource(R.string.connecting),
					style = CustomTypography.buttonMain,
					color = MaterialTheme.colorScheme.onSurface,
				)
			},
			color = Color(0xFFB0ADB6),
			modifier = buttonModifier,
			shape = RoundedCornerShape(50),
		)

		ConnectionState.Disconnecting -> MainStyledButton(
			onClick = {},
			content = {
				Text(
					stringResource(R.string.disconnecting),
					style = CustomTypography.buttonMain,
					color = MaterialTheme.colorScheme.onSurface,
				)
			},
			color = Color(0xFFB0ADB6),
			modifier = buttonModifier,
			shape = RoundedCornerShape(50),
		)

		ConnectionState.Connected -> MainStyledButton(
			onClick = onDisconnect,
			content = {
				Text(
					stringResource(R.string.disconnect),
					style = CustomTypography.buttonMain,
					color = MaterialTheme.colorScheme.onSurface,
				)
			},
			color = Color.Transparent,
			borderStroke = BorderStroke(1.dp, MaterialTheme.colorScheme.outline),
			modifier = buttonModifier,
			shape = RoundedCornerShape(50),
		)

		is ConnectionState.Error -> {
			val isSubscriptionError =
				connectionState.reason is ErrorStateReason.InactiveSubscription ||
					connectionState.reason is ErrorStateReason.InactiveAccount
			val isAccountActionPending =
				accountState == AccountControllerState.Syncing ||
					accountState == AccountControllerState.PendingSubscription

			when {
				isSubscriptionError && !isAccountActionPending && isVpnAlwaysOn(context) ->
					MainStyledButton(
						onClick = onStopKillSwitch,
						content = {
							Text(
								stringResource(R.string.stop),
								style = CustomTypography.buttonMain,
								color = MaterialTheme.colorScheme.onError,
							)
						},
						color = MaterialTheme.colorScheme.error,
						modifier = buttonModifier,
						shape = RoundedCornerShape(50),
					)
				isSubscriptionError && !isAccountActionPending ->
					MainStyledButton(
						onClick = onGetStartedClick,
						content = {
							Text(
								stringResource(R.string.get_started),
								style = CustomTypography.buttonMain,
							)
						},
						modifier = buttonModifier,
						shape = RoundedCornerShape(50),
					)
				else ->
					MainStyledButton(
						onClick = if (isMnemonicStored) onConnect else onGetStartedClick,
						content = {
							Text(
								stringResource(if (isMnemonicStored) R.string.connect else R.string.get_started),
								style = CustomTypography.buttonMain,
							)
						},
						modifier = buttonModifier,
						shape = RoundedCornerShape(50),
					)
			}
		}

		is ConnectionState.StartFailure -> MainStyledButton(
			onClick = onConnect,
			content = {
				Text(
					stringResource(R.string.connect),
					style = CustomTypography.buttonMain,
				)
			},
			modifier = buttonModifier,
			shape = RoundedCornerShape(50),
		)
	}
}

@Preview(name = "Disconnected – dark", uiMode = Configuration.UI_MODE_NIGHT_YES)
@Composable
private fun PreviewDisconnectedDark() {
	NymVPNTheme(Theme.DARK_MODE) {
		ConnectPanel(
			connectionState = ConnectionState.Disconnected,
			accountState = AccountControllerState.ReadyToConnect,
			isMnemonicStored = true,
			vpnMode = Tunnel.Mode.FIVE_HOP_MIXNET,
			exitNode = ServerNode(name = "169.128.6.931", countryCode = "fr", location = "France"),
			entryNode = ServerNode(name = "169.128.6.932", countryCode = "fr", location = "France"),
			onFastModeClick = {},
			onAnonModeClick = {},
			onConnect = {},
			onDisconnect = {},
			onStopKillSwitch = {},
			onGetStartedClick = {},
			onPanelStateChange = {},
			initialPanelState = PanelState.COLLAPSED,
			onExitNodeClick = {},
			onEntryNodeClick = {},
		)
	}
}
