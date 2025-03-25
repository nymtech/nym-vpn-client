package net.nymtech.nymvpn.ui.screens.settings.components

import androidx.compose.foundation.clickable
import androidx.compose.foundation.interaction.MutableInteractionSource
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.material3.MaterialTheme
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.remember
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.ClipboardManager
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.text.AnnotatedString
import androidx.compose.ui.text.style.TextOverflow
import net.nymtech.nymvpn.R

@Composable
fun AccountId(clipboardManager: ClipboardManager, accountId: String) {
	Column(
		verticalArrangement = Arrangement.Bottom,
		horizontalAlignment = Alignment.Start,
		modifier = Modifier
			.fillMaxSize(),
	) {
		Text(
			stringResource(R.string.account_id) + " $accountId",
			style = MaterialTheme.typography.bodyMedium,
			color = MaterialTheme.colorScheme.secondary,
			maxLines = 1,
			overflow = TextOverflow.Ellipsis,
			modifier = Modifier.clickable(
				indication = null,
				interactionSource = remember { MutableInteractionSource() },
			) {
				clipboardManager.setText(AnnotatedString(accountId))
			},
		)
	}
}
