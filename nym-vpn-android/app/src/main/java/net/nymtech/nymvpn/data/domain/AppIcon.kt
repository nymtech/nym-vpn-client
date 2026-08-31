package net.nymtech.nymvpn.data.domain

import androidx.annotation.DrawableRes
import androidx.annotation.StringRes
import net.nymtech.nymvpn.R

enum class AppIcon(val componentName: String, @DrawableRes val previewDrawable: Int, @StringRes val labelRes: Int) {
	DEFAULT(
		componentName = "net.nymtech.nymvpn.ui.MainActivityDefault",
		previewDrawable = R.mipmap.ic_launcher,
		labelRes = R.string.app_icon_label_default,
	),
	DARK(
		componentName = "net.nymtech.nymvpn.ui.MainActivityDark",
		previewDrawable = R.mipmap.ic_launcher_dark,
		labelRes = R.string.app_icon_label_dark,
	),
	LIGHT(
		componentName = "net.nymtech.nymvpn.ui.MainActivityLight",
		previewDrawable = R.mipmap.ic_launcher_light,
		labelRes = R.string.app_icon_label_light,
	),
	CALCULATOR(
		componentName = "net.nymtech.nymvpn.ui.MainActivityCalculator",
		previewDrawable = R.mipmap.ic_launcher_calculator,
		labelRes = R.string.app_icon_label_calculator,
	),
	NOTES(
		componentName = "net.nymtech.nymvpn.ui.MainActivityNotes",
		previewDrawable = R.mipmap.ic_launcher_notes,
		labelRes = R.string.app_icon_label_notes,
	),
}
