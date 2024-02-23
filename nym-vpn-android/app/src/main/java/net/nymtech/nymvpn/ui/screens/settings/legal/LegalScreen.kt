package net.nymtech.nymvpn.ui.screens.settings.legal

import android.content.Intent
import android.net.Uri
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.common.buttons.SelectionItem
import net.nymtech.nymvpn.ui.common.buttons.SurfaceSelectionGroupButton
import net.nymtech.nymvpn.util.scaledHeight
import net.nymtech.nymvpn.util.scaledWidth
import timber.log.Timber

@Composable
fun LegalScreen() {

    val context = LocalContext.current
    fun openWebPage(url: String) {
        try {
            val webpage: Uri = Uri.parse(url)
            val intent = Intent(Intent.ACTION_VIEW, webpage)
            context.startActivity(intent)
        } catch (e: Exception) {
            Timber.e("Failed to launch webpage")
        }
    }

  Column(
      horizontalAlignment = Alignment.Start,
      verticalArrangement = Arrangement.spacedBy(24.dp, Alignment.Top),
      modifier =
          Modifier.verticalScroll(rememberScrollState())
              .fillMaxSize()
              .padding(top = 24.dp.scaledHeight())
              .padding(horizontal = 24.dp.scaledWidth())) {
        SurfaceSelectionGroupButton(
            listOf(
                SelectionItem(title = stringResource(R.string.terms_of_use), onClick = {}),
                SelectionItem(
                    title = stringResource(R.string.privacy_policy),
                    onClick = { openWebPage(context.getString(R.string.privacy_link)) }),
                SelectionItem(title = stringResource(R.string.licenses), onClick = {})))
      }
}
