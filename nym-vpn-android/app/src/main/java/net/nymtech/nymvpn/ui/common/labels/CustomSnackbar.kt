package net.nymtech.nymvpn.ui.common.labels

import androidx.compose.foundation.layout.Arrangement
import androidx.compose.foundation.layout.Box
import androidx.compose.foundation.layout.Row
import androidx.compose.foundation.layout.fillMaxSize
import androidx.compose.foundation.layout.padding
import androidx.compose.material3.Snackbar
import androidx.compose.material3.Text
import androidx.compose.runtime.Composable
import androidx.compose.runtime.CompositionLocalProvider
import androidx.compose.ui.Alignment
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.Color
import androidx.compose.ui.platform.LocalLayoutDirection
import androidx.compose.ui.unit.LayoutDirection
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.ui.theme.CustomColors
import net.nymtech.nymvpn.util.scaledHeight
import net.nymtech.nymvpn.util.scaledWidth

@Composable
fun CustomSnackBar(
    message: String,
    isRtl: Boolean = true,
    containerColor: Color = CustomColors.snackBarBackgroundColor
) {
    Box(
        modifier = Modifier
            .fillMaxSize()
            .padding(horizontal = 24.dp.scaledWidth())
            .padding(top = 100.dp.scaledHeight()), contentAlignment = Alignment.TopCenter
    ) {
        Snackbar(containerColor = containerColor) {
            CompositionLocalProvider(
                LocalLayoutDirection provides
                        if (isRtl) LayoutDirection.Rtl else LayoutDirection.Ltr
            ) {
                Row(verticalAlignment = Alignment.Top, horizontalArrangement = Arrangement.Center) {
                    Text(message, color = CustomColors.snackbarTextColor)
                }
            }
        }
    }
}