package net.nymtech.nymvpn.ui.screens.account.delete.components

import android.content.res.Configuration
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.Spacer
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.fillMaxWidth
import androidx.compose.foundation.layout.height
import androidx.compose.foundation.layout.navigationBarsPadding
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Warning
import androidx.compose.material3.Icon
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
import net.nymtech.nymvpn.ui.common.buttons.MainStyledButton
import net.nymtech.nymvpn.ui.common.buttons.TransparentButton
import net.nymtech.nymvpn.ui.theme.LocalNymColors
import net.nymtech.nymvpn.ui.theme.NymVPNTheme
import net.nymtech.nymvpn.ui.theme.Theme
import net.nymtech.nymvpn.util.extensions.scaledHeight
import net.nymtech.nymvpn.util.extensions.scaledWidth

@Composable
fun PlayStopContent(onOpenPlayClick: () -> Unit, onKeepClick: () -> Unit) {
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
			Icon(
				imageVector = Icons.Outlined.Warning,
				contentDescription = null,
				tint = LocalNymColors.current.warning,
				modifier = Modifier.size(48.dp),
			)
			Text(
				text = stringResource(R.string.delete_account_play_title),
				style = MaterialTheme.typography.titleLarge,
				color = MaterialTheme.colorScheme.onPrimaryContainer,
				textAlign = TextAlign.Center,
			)
			Text(
				text = stringResource(R.string.delete_account_play_descr_1),
				style = MaterialTheme.typography.bodyMedium,
				color = MaterialTheme.colorScheme.onBackground,
				textAlign = TextAlign.Center,
			)
			Text(
				text = stringResource(R.string.delete_account_play_descr_2),
				style = MaterialTheme.typography.bodySmall,
				color = MaterialTheme.colorScheme.onBackground,
				textAlign = TextAlign.Center,
			)
		}
		MainStyledButton(
			onClick = onOpenPlayClick,
			content = {
				Text(
					stringResource(R.string.delete_account_play_button),
					style = MaterialTheme.typography.bodyLarge,
				)
			},
			modifier = Modifier
				.fillMaxWidth()
				.height(48.dp.scaledHeight()),
		)
		Spacer(Modifier.height(8.dp))
		TransparentButton(
			onClick = onKeepClick,
			content = {
				Text(
					stringResource(R.string.delete_account_keep_button),
					style = MaterialTheme.typography.bodyLarge,
					color = MaterialTheme.colorScheme.onPrimaryContainer,
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
internal fun PreviewPlayStopContent() {
	NymVPNTheme(Theme.default()) {
		PlayStopContent(onOpenPlayClick = {}, onKeepClick = {})
	}
}
