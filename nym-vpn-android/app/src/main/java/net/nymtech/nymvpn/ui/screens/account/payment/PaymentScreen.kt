package net.nymtech.nymvpn.ui.screens.account.payment

import android.content.res.Configuration
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxHeight
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.LaunchedEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableIntStateOf
import androidx.compose.runtime.remember
import androidx.compose.runtime.setValue
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import androidx.hilt.navigation.compose.hiltViewModel
import androidx.lifecycle.compose.collectAsStateWithLifecycle
import kotlinx.coroutines.delay
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.Route
import net.nymtech.nymvpn.ui.common.animations.PulsingDotsWave
import net.nymtech.nymvpn.ui.common.navigation.LocalNavController
import net.nymtech.nymvpn.ui.theme.CustomColors
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.ui.theme.Typography
import net.nymtech.nymvpn.util.extensions.replaceCurrentWith

@Composable
fun PaymentScreen(viewModel: PaymentViewModel = hiltViewModel()) {
	val success by viewModel.success.collectAsStateWithLifecycle(null)
	val navController = LocalNavController.current

	LaunchedEffect(success) {
		if (success == false) {
		}
	}

	PaymentScreen {
		if (success == true) {
			navController.replaceCurrentWith(Route.SelectPlan)
		}
	}
}

@Composable
fun PaymentScreen(onAnimationEnd: () -> Unit) {
	var step by remember { mutableIntStateOf(0) }

	LaunchedEffect(Unit) {
		delay(2000)
		step = 1
		delay(2000)
		step = 2
		delay(2000)
		step = 3
		onAnimationEnd()
	}

	Column(
		modifier = Modifier
			.fillMaxSize()
			.background(MaterialTheme.colorScheme.background),
		horizontalAlignment = Alignment.CenterHorizontally,
	) {
		Row(
			modifier = Modifier
				.padding(horizontal = 16.dp)
				.fillMaxWidth()
				.height(4.dp),
			horizontalArrangement = Arrangement.spacedBy(4.dp),
		) {
			Box(
				modifier = Modifier
					.weight(1f)
					.fillMaxHeight()
					.background(
						MaterialTheme.colorScheme.primary,
						shape = RoundedCornerShape(size = 4.dp),
					),
			)
			Box(
				modifier = Modifier
					.weight(1f)
					.fillMaxHeight()
					.background(
						if (step > 1) {
							MaterialTheme.colorScheme.primary
						} else {
							MaterialTheme.colorScheme.surfaceContainer
						},
						shape = RoundedCornerShape(size = 4.dp),
					),
			)
		}

		Column(
			horizontalAlignment = Alignment.CenterHorizontally,
			modifier = Modifier.padding(top = 200.dp),
		) {
			Box(
				modifier = Modifier
					.size(56.dp)
					.background(
						color = CustomColors.iconBackground,
						shape = RoundedCornerShape(size = 8.dp),
					)
					.border(
						width = 1.dp,
						color = CustomColors.iconBorder,
						shape = RoundedCornerShape(size = 8.dp),
					),
			) {
				PulsingDotsWave(
					modifier = Modifier
						.align(Alignment.Center)
						.padding(8.dp),
				)
			}

			val data = when(step) {
				0 -> {
					Pair(R.string.account_payment_processing, R.string.account_payment_verifying)
				}
				1 -> {
					Pair(R.string.account_payment_retrieving, R.string.account_payment_credentials)
				}
				2 -> {
					Pair(R.string.account_payment_saving, R.string.account_payment_anonymous)
				}
				3 -> {
					Pair(R.string.account_payment_welcome, R.string.account_payment_protected)
				}
				else -> {
					Pair(-1, -1)
				}
			}

			Text(
				text = stringResource(data.first),
				style = Typography.titleMedium,
				textAlign = TextAlign.Center,
				color = MaterialTheme.colorScheme.onBackground,
				modifier = Modifier.padding(top = 16.dp, start = 40.dp, end = 40.dp),
				fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
			)

			Text(
				text = stringResource(data.second),
				style = Typography.bodyMedium,
				textAlign = TextAlign.Center,
				modifier = Modifier.padding(top = 16.dp, start = 40.dp, end = 40.dp),
				color = MaterialTheme.colorScheme.outline,
				fontFamily = FontFamily(Font(R.font.lab_grotesque_regular)),
			)
		}
	}
}

@Preview(uiMode = Configuration.UI_MODE_NIGHT_YES, showBackground = true)
@Composable
private fun PreviewPaymentScreen() {
	NymVPNTheme(Theme.default()) {
		PaymentScreen(onAnimationEnd = {})
	}
}
