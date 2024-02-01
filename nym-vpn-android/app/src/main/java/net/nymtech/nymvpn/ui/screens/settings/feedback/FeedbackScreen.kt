package net.nymtech.nymvpn.ui.screens.settings.feedback

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.theme.screenPadding

@Composable
fun FeedbackScreen() {

  Column(
      horizontalAlignment = Alignment.Start,
      verticalArrangement = Arrangement.spacedBy(screenPadding, Alignment.Top),
      modifier = Modifier.fillMaxSize().padding(top = screenPadding).padding(horizontal = screenPadding)) {
        Text(stringResource(id = R.string.feedback))
      }
}