package net.nymtech.nymvpn.ui.screens.main.components

import androidx.compose.animation.AnimatedVisibility
import androidx.compose.foundation.isSystemInDarkTheme
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
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
import net.nymtech.nymvpn.ui.model.StateMessage.Error
import net.nymtech.nymvpn.ui.model.StateMessage.StartError
import net.nymtech.nymvpn.ui.theme.CustomColors
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.util.extensions.scaledHeight
import net.nymtech.nymvpn.util.extensions.toUserMessage
import net.nymtech.vpn.backend.Tunnel

@Composable
fun ConnectionStatus(
	connectionState: ConnectionState,
	vpnMode: Tunnel.Mode,
	stateMessage: StateMessage,
	connectionTime: String?,
	theme: Theme,
	modifier: Modifier = Modifier,
) {
	val isDarkMode = isSystemInDarkTheme()
	val animation by remember(theme) {
		val asset = when (theme) {
			Theme.AUTOMATIC, Theme.DYNAMIC -> if (isDarkMode) {
				if (vpnMode.isTwoHop()) R.raw.noise_2hop_dark else R.raw.noise_5hop_dark
			} else if (vpnMode.isTwoHop()) R.raw.noise_2hop_light else R.raw.noise_5hop_light
			Theme.DARK_MODE -> if (vpnMode.isTwoHop()) R.raw.noise_2hop_dark else R.raw.noise_5hop_dark
			Theme.LIGHT_MODE -> if (vpnMode.isTwoHop()) R.raw.noise_2hop_light else R.raw.noise_5hop_light
		}
		mutableStateOf(asset)
	}
	val composition = rememberLottieComposition(LottieCompositionSpec.RawRes(animation))

	Column(
		verticalArrangement = Arrangement.spacedBy(8.dp.scaledHeight()),
		horizontalAlignment = Alignment.CenterHorizontally,
		modifier = modifier.padding(top = 56.dp.scaledHeight()),
	) {
		AnimatedVisibility(visible = connectionState == ConnectionState.Connected) {
			val logoAnimationState = animateLottieCompositionAsState(
				composition = composition.value,
				speed = 1f,
				isPlaying = connectionState == ConnectionState.Connected,
				iterations = LottieConstants.IterateForever,
				cancellationBehavior = LottieCancellationBehavior.Immediately,
			)
			LottieAnimation(
				composition = composition.value,
				progress = { logoAnimationState.progress },
			)
		}
		ConnectionStateDisplay(connectionState = connectionState, theme = theme)
		when (stateMessage) {
			is StateMessage.Status -> StatusInfoLabel(
				message = stateMessage.message.asString(LocalContext.current),
				textColor = MaterialTheme.colorScheme.onSurfaceVariant,
			)
			is Error -> StatusInfoLabel(
				message = stateMessage.reason.toUserMessage(LocalContext.current),
				textColor = CustomColors.error,
			)
			is StartError -> StatusInfoLabel(
				message = stateMessage.exception.toUserMessage(LocalContext.current),
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
