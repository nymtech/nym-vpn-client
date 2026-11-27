package net.nymtech.nymvpn.ui.screens.settings.components

import androidx.annotation.StringRes
import net.nymtech.nymvpn.R

/** Settings sections in display order; a null [title] means no header. */
enum class SettingsSection(@StringRes val title: Int?) {
	ACCOUNT(R.string.settings_section_account),
	PROTECTION(R.string.settings_section_protection),
	CONNECTION(R.string.settings_section_connection),
	ROUTING(R.string.settings_section_routing),
	APP(R.string.settings_section_app),
	HELP_DATA(R.string.settings_section_help_data),
	FOOTER(null),
}
