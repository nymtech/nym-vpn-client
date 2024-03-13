package net.nymtech.nymvpn.ui.common.functions

import androidx.compose.foundation.Image
import androidx.compose.foundation.layout.padding
import androidx.compose.foundation.layout.size
import androidx.compose.material3.MaterialTheme
import androidx.compose.runtime.Composable
import androidx.compose.ui.Modifier
import androidx.compose.ui.graphics.ColorFilter
import androidx.compose.ui.graphics.vector.ImageVector
import androidx.compose.ui.platform.LocalContext
import androidx.compose.ui.res.vectorResource
import androidx.compose.ui.unit.dp
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.model.Country
import net.nymtech.nymvpn.ui.theme.iconSize
import net.nymtech.nymvpn.util.StringUtils
import net.nymtech.nymvpn.util.scaledHeight
import net.nymtech.nymvpn.util.scaledWidth

@Composable
fun countryIcon(country: Country): @Composable () -> Unit {
    val context = LocalContext.current
    val image =
        if (country.isFastest) ImageVector.vectorResource(R.drawable.bolt)
        else ImageVector.vectorResource(StringUtils.getFlagImageVectorByName(context, country.isoCode.lowercase()))
    return {
        Image(
            image,
            image.name,
            modifier = Modifier
                .padding(horizontal = 16.dp.scaledWidth(), vertical = 16.dp.scaledHeight())
                .size(
                    iconSize
                ),
            colorFilter =
            if (country.isFastest) ColorFilter.tint(MaterialTheme.colorScheme.onSurface)
            else null)
    }
}