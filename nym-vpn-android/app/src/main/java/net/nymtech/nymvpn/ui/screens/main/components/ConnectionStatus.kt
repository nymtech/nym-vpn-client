package net.nymtech.nymvpn.ui.screens.main.components

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.State
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import com.airbnb.lottie.compose.LottieAnimation
import com.airbnb.lottie.compose.LottieCancellationBehavior
import com.airbnb.lottie.compose.LottieCompositionSpec
import com.airbnb.lottie.compose.LottieConstants
import com.airbnb.lottie.compose.animateLottieCompositionAsState
import com.airbnb.lottie.compose.rememberLottieComposition
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.common.labels.StatusInfoLabel
import net.nymtech.nymvpn.ui.model.ConnectionState
import net.nymtech.nymvpn.ui.model.StateMessage
import net.nymtech.nymvpn.ui.theme.CustomColors
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.util.extensions.isVpnAlwaysOn
import net.nymtech.nymvpn.util.extensions.scaledHeight
import net.nymtech.nymvpn.util.extensions.toUserMessage
import net.nymtech.vpn.backend.Tunnel
import nym_vpn_lib_types.ErrorStateReason

@Composable
fun ConnectionStatus(
	connectionState: ConnectionState,
	vpnMode: Tunnel.Mode,
	stateMessage: StateMessage,
	connectionTime: String?,
	theme: Theme,
	modifier: Modifier = Modifier,
	isAppInForeground: Boolean,
) {
	val isDarkMode = isSystemInDarkTheme()
	val surfaceAvailable by rememberSurfaceAvailability()
	val context = LocalContext.current

	val canPlayAnimation = connectionState == ConnectionState.Connected && isAppInForeground && surfaceAvailable

	val animation by remember(theme) {
		val asset = when (theme) {
			Theme.AUTOMATIC, Theme.DYNAMIC -> if (isDarkMode) {
				if (vpnMode.isTwoHop()) R.raw.noise_2hop_dark else R.raw.noise_5hop_dark
			} else if (vpnMode.isTwoHop()) R.raw.noise_2hop_light else R.raw.noise_5hop_light

			Theme.DARK_MODE -> if (vpnMode.isTwoHop()) R.raw.noise_2hop_dark else R.raw.noise_5hop_dark
			Theme.LIGHT_MODE -> if (vpnMode.isTwoHop()) R.raw.noise_2hop_light else R.raw.noise_5hop_light
		}
		mutableIntStateOf(asset)
	}

	val composition = rememberLottieComposition(LottieCompositionSpec.RawRes(animation))

	Column(
		verticalArrangement = Arrangement.spacedBy(8.dp.scaledHeight()),
		horizontalAlignment = Alignment.CenterHorizontally,
		modifier = modifier.padding(top = 56.dp.scaledHeight()),
	) {
		AnimatedVisibility(visible = canPlayAnimation) {
			val logoAnimationState = animateLottieCompositionAsState(
				composition = composition.value,
				speed = 1f,
				isPlaying = canPlayAnimation,
				iterations = LottieConstants.IterateForever,
				cancellationBehavior = LottieCancellationBehavior.Immediately,
			)
			LottieAnimation(
				composition = composition.value,
				progress = { logoAnimationState.progress },
			)
		}

		ConnectionStateDisplay(connectionState = connectionState, stateMessage, theme = theme)

		when (stateMessage) {
			is StateMessage.Status -> StatusInfoLabel(
				message = stateMessage.message.asString(context),
				textColor = MaterialTheme.colorScheme.onSurfaceVariant,
			)

			is StateMessage.Error ->
				when (stateMessage.reason) {
					ErrorStateReason.InactiveSubscription -> {
						if (isVpnAlwaysOn(context)) {
							StatusInfoLabel(
								message = context.getString(R.string.error_kill_switch),
								textColor = CustomColors.error,
							)
						} else {
							StatusInfoLabel(
								message = stateMessage.reason.toUserMessage(context),
								textColor = CustomColors.error,
							)
						}
					}

					else -> {
						StatusInfoLabel(
							message = stateMessage.reason.toUserMessage(context),
							textColor = CustomColors.error,
						)
					}
				}

			is StateMessage.StartError -> StatusInfoLabel(
				message = stateMessage.exception.toUserMessage(context),
				textColor = CustomColors.error,
			)
		}

		AnimatedVisibility(visible = connectionTime != null) {
			connectionTime?.let {
				StatusInfoLabel(
					message = it,
					textColor = MaterialTheme.colorScheme.onSurface,
				)
			}
		}
	}
}

@Composable
private fun rememberSurfaceAvailability(): State<Boolean> {
	val surfaceAvailable = remember { mutableStateOf(true) }
	androidx.compose.ui.platform.LocalView.current.viewTreeObserver
		.addOnWindowAttachListener(
			object : android.view.ViewTreeObserver.OnWindowAttachListener {
				override fun onWindowAttached() {
					surfaceAvailable.value = true
				}

				override fun onWindowDetached() {
					surfaceAvailable.value = false
				}
			},
		)
	return surfaceAvailable
}

@Composable
@Preview
private fun ConnectionStatusPreview() {
	NymVPNTheme(Theme.default()) {
		ConnectionStatus(
			ConnectionState.Disconnected,
			Tunnel.Mode.TWO_HOP_MIXNET,
			StateMessage.Error(ErrorStateReason.InactiveSubscription),
			null,
			Theme.default(),
			isAppInForeground = false,
		)
	}
}
