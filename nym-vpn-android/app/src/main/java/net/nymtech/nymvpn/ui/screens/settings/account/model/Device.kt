package net.nymtech.nymvpn.ui.screens.settings.account.model

import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.util.StringValue

typealias Devices = List<Device>

data class Device(
	val name: String,
	val type: DeviceType,
)

enum class DeviceType {
	MAC_OS,
	IOS,
	ANDROID,
	WINDOWS,
	LINUX,
	;

	fun formattedName(): StringValue {
		return when (this) {
			ANDROID -> StringValue.StringResource(R.string.android)
			IOS -> StringValue.StringResource(R.string.ios)
			WINDOWS -> StringValue.StringResource(R.string.windows)
			MAC_OS -> StringValue.StringResource(R.string.macos)
			LINUX -> StringValue.StringResource(R.string.linux)
		}
	}

	fun icon(): Int {
		return when (this) {
			ANDROID, IOS -> R.drawable.phone
			WINDOWS, MAC_OS, LINUX -> R.drawable.laptop
		}
	}
}
