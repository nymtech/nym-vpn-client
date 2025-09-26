package net.nymtech.nymvpn.ui.screens.account.generating

import android.util.Log
import android.content.res.Configuration
import androidx.compose.foundation.background
import androidx.compose.foundation.border
import androidx.compose.foundation.layout.*
import androidx.compose.foundation.shape.RoundedCornerShape
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.*
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.font.Font
import androidx.compose.ui.text.font.FontFamily
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import kotlinx.coroutines.delay
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.screens.account.generating.components.PulsingDotsWave
import net.nymtech.nymvpn.ui.theme.CustomColors
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.ui.theme.Typography


@Composable
fun GeneratingScreen() {
	var step by remember { mutableIntStateOf(0) }

	LaunchedEffect(Unit) {
		delay(3000)
		step = 1
		delay(3000)
		Log.d("KeyGenerationScreen", "Ready")
	}

	Column(
		modifier = Modifier
			.fillMaxSize()
			.background(MaterialTheme.colorScheme.background)
			.padding(WindowInsets.systemBars.asPaddingValues()),
		horizontalAlignment = Alignment.CenterHorizontally
	) {
		Row(
			modifier = Modifier
				.padding(horizontal = 16.dp, vertical = 24.dp)
				.fillMaxWidth()
				.height(4.dp),
			horizontalArrangement = Arrangement.spacedBy(4.dp)
		) {
			Box(
				modifier = Modifier
					.weight(1f)
					.fillMaxHeight()
					.background(Color(0xFF00E676))
			)
			Box(
				modifier = Modifier
					.weight(1f)
					.fillMaxHeight()
					.background(if(step > 0) Color(0xFF00E676) else Color(0xFF2C2C2C))
			)
		}
		Column(
		) {
			Box(modifier = Modifier
				.size(56.dp)
				.background(color = CustomColors.iconBackground)
				.border(width = 1.dp,
					color = Color(0x4014E76F),
					shape = RoundedCornerShape(size = 8.dp))
			) {
				PulsingDotsWave(
					modifier = Modifier
						.align(Alignment.Center)
						.padding(8.dp)
				)
			}

			Column(horizontalAlignment = Alignment.CenterHorizontally) {
				val title = if(step == 0) R.string.account_generating_creating else R.string.account_generating_securing
				val text = if(step == 0) R.string.account_generating_establishing else R.string.account_generating_encrypting
				Text(
					text = stringResource(title),
					style = Typography.titleMedium,
					color = MaterialTheme.colorScheme.onBackground,
					fontFamily = FontFamily(Font(R.font.lab_grotesque_regular))
				)
				Spacer(Modifier.height(8.dp))
				Text(
					text = stringResource(text),
					style = Typography.bodyMedium,
					color = MaterialTheme.colorScheme.outline,
					fontFamily = FontFamily(Font(R.font.lab_grotesque_regular))
				)
			}
		}
	}
}

@Preview(uiMode = Configuration.UI_MODE_NIGHT_YES, showBackground = true)
@Composable
fun PreviewGeneratingScreen() {
	NymVPNTheme(Theme.default()) {
		GeneratingScreen()
	}
}
