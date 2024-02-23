package net.nymtech.nymvpn.ui.screens.settings.logs

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
import net.nymtech.nymvpn.util.scaledHeight
import net.nymtech.nymvpn.util.scaledWidth

@Composable
fun LogsScreen() {

  Column(
      horizontalAlignment = Alignment.Start,
      verticalArrangement = Arrangement.spacedBy(24.dp.scaledHeight(), Alignment.Top),
      modifier = Modifier.fillMaxSize().padding(top = 24.dp.scaledHeight()).padding(horizontal = 24.dp.scaledWidth())) {
        Text(stringResource(id = R.string.logs))
      }
}