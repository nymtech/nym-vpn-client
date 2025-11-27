package net.nymtech.nymvpn.data

import net.nymtech.nymvpn.ui.screens.settings.tunneling.AppInfo

interface SplitTunnelingRepository {

	suspend fun saveAppInfoList(appInfo: List<AppInfo>)

	suspend fun getAppInfoList(): List<AppInfo>
}
