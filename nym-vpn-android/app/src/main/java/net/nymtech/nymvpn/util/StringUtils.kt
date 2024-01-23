package net.nymtech.nymvpn.util

import android.content.Context
import androidx.compose.ui.text.buildAnnotatedString
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.model.Country

object StringUtils {
    fun buildCountryNameString(country : Country, context : Context) : String {
        return buildAnnotatedString {
            if(country.isFastest) {
                append(context.getString(R.string.fastest))
                append(" (")
                append(country.name)
                append(")")}
            else append(country.name)
        }.text
    }
    fun getImageVectorByName(context: Context, name: String): Int {
        return context.resources.getIdentifier(name, "drawable", context.packageName)
    }
}