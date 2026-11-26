package net.nymtech.nymvpn.ui.screens.settings.privacy

import net.nymtech.nymvpn.ui.screens.settings.privacy.components.LoggingLevel

/** The three data choices shown on Data, privacy & logs, read from settings and the core config. */
data class PrivacyChoices(val statsEnabled: Boolean, val sentryEnabled: Boolean, val loggingLevel: LoggingLevel)
