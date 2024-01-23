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

@Composable
fun FeedbackScreen() {

  val padding = 24.dp

  Column(
      horizontalAlignment = Alignment.Start,
      verticalArrangement = Arrangement.spacedBy(padding, Alignment.Top),
      modifier = Modifier.fillMaxSize().padding(top = padding).padding(horizontal = padding)) {
        Text(stringResource(id = R.string.feedback))
      }
}