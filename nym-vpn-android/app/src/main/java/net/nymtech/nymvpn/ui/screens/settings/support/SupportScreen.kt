package net.nymtech.nymvpn.ui.screens.settings.support

import android.content.Intent
import android.net.Uri
import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Column
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.rememberScrollState
import androidx.compose.foundation.verticalScroll
import androidx.compose.material.icons.Icons
import androidx.compose.material.icons.outlined.Email
import androidx.compose.runtime.Composable
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.stringResource
import androidx.compose.ui.res.vectorResource
import androidx.compose.ui.unit.dp
import androidx.core.content.ContextCompat
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.common.buttons.SelectionItem
import net.nymtech.nymvpn.ui.common.buttons.SurfaceSelectionGroupButton
import net.nymtech.nymvpn.ui.theme.screenPadding
import net.nymtech.nymvpn.util.Constants
import timber.log.Timber

@Composable
fun SupportScreen() {
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

    fun launchEmail() {
        try {
            val intent =
                Intent(Intent.ACTION_SENDTO).apply {
                    type = Constants.EMAIL_MIME_TYPE
                    putExtra(Intent.EXTRA_EMAIL, arrayOf(context.getString(R.string.support_email)))
                    putExtra(Intent.EXTRA_SUBJECT, context.getString(R.string.email_subject))
                }
            ContextCompat.startActivity(
                context,
                Intent.createChooser(intent, context.getString(R.string.email_chooser)),
                null,
            )
        } catch (e: Exception) {
            //TODO handle exception like no email client
        }
    }

    Column(
        horizontalAlignment = Alignment.Start,
        verticalArrangement = Arrangement.spacedBy(24.dp, Alignment.Top),
        modifier =
        Modifier.verticalScroll(rememberScrollState())
            .fillMaxSize()
            .padding(top = screenPadding)
            .padding(horizontal = screenPadding)) {
        SurfaceSelectionGroupButton(
            listOf(
                SelectionItem(
                    leadingIcon = ImageVector.vectorResource(R.drawable.faq),
                    title = stringResource(R.string.check_faq),
                    onClick = { openWebPage(context.getString(R.string.faq_link)) }),
            ))
        SurfaceSelectionGroupButton(
            listOf(
                SelectionItem(
                    leadingIcon = Icons.Outlined.Email,
                    title = stringResource(R.string.send_email),
                    onClick = {
                        launchEmail()
                    }),
            ))
        SurfaceSelectionGroupButton(
            listOf(
                SelectionItem(
                    leadingIcon = ImageVector.vectorResource(R.drawable.matrix),
                    title = stringResource(R.string.join_matrix),
                    onClick = {
                        openWebPage(context.getString(R.string.matrix_url))
                    }),
            ))
        SurfaceSelectionGroupButton(
            listOf(
                SelectionItem(
                    leadingIcon = ImageVector.vectorResource(R.drawable.discord),
                    title = stringResource(R.string.join_discord),
                    onClick = {
                        openWebPage(context.getString(R.string.discord_url))
                    }),
            ))
    }
}