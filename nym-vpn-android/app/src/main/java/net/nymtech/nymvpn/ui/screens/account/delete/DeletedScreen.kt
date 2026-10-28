package net.nymtech.nymvpn.ui.screens.account.delete

import android.content.res.Configuration
import androidx.activity.compose.BackHandler
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.style.TextAlign
import androidx.compose.ui.tooling.preview.Preview
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.Route
import net.nymtech.nymvpn.ui.common.buttons.MainStyledButton
import net.nymtech.nymvpn.ui.common.navigation.LocalNavController
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.util.extensions.scaledHeight
import net.nymtech.nymvpn.util.extensions.scaledWidth

@Composable
fun DeletedScreen() {
	val navController = LocalNavController.current

	val onDone = {
		navController.navigate(Route.Main()) {
			popUpTo(0) { inclusive = true }
			launchSingleTop = true
		}
	}

	BackHandler(onBack = onDone)

	DeletedScreen(onDoneClick = onDone)
}

@Composable
fun DeletedScreen(onDoneClick: () -> Unit) {
	Column(
		horizontalAlignment = Alignment.CenterHorizontally,
		modifier = Modifier
			.fillMaxSize()
			.padding(horizontal = 24.dp.scaledWidth(), vertical = 24.dp.scaledHeight())
			.navigationBarsPadding(),
	) {
		Column(
			horizontalAlignment = Alignment.CenterHorizontally,
			verticalArrangement = Arrangement.spacedBy(16.dp, Alignment.CenterVertically),
			modifier = Modifier
				.weight(1f)
				.fillMaxWidth(),
		) {
			Text(
				text = stringResource(R.string.delete_account_confirm_title),
				style = MaterialTheme.typography.titleLarge,
				color = MaterialTheme.colorScheme.onPrimaryContainer,
				textAlign = TextAlign.Center,
			)
			Text(
				text = stringResource(R.string.delete_account_confirm_descr_1),
				style = MaterialTheme.typography.bodyMedium,
				color = MaterialTheme.colorScheme.onBackground,
				textAlign = TextAlign.Center,
			)
			Text(
				text = stringResource(R.string.delete_account_confirm_descr_2),
				style = MaterialTheme.typography.bodySmall,
				color = MaterialTheme.colorScheme.onBackground,
				textAlign = TextAlign.Center,
			)
		}
		MainStyledButton(
			onClick = onDoneClick,
			content = {
				Text(
					stringResource(R.string.button_done),
					style = MaterialTheme.typography.bodyLarge,
				)
			},
			modifier = Modifier
				.fillMaxWidth()
				.height(48.dp.scaledHeight()),
		)
	}
}

@Composable
@Preview(uiMode = Configuration.UI_MODE_NIGHT_YES)
internal fun PreviewDeletedScreen() {
	NymVPNTheme(Theme.default()) {
		DeletedScreen(onDoneClick = {})
	}
}
