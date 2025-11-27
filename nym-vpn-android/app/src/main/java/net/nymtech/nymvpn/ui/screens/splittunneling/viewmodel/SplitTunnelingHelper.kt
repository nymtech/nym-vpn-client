package net.nymtech.nymvpn.ui.screens.splittunneling.viewmodel

import android.Manifest
import android.content.pm.ApplicationInfo
import android.content.pm.PackageManager
import net.nymtech.nymvpn.BuildConfig
import net.nymtech.nymvpn.data.SplitTunnelingRepository
import net.nymtech.nymvpn.ui.screens.splittunneling.model.AppFilter
import net.nymtech.nymvpn.ui.screens.splittunneling.model.AppInfo
import javax.inject.Inject

class SplitTunnelingHelper @Inject constructor() {

	/**
	 * A set of package names for common system apps that are not useful for split tunneling.
	 * These apps typically do not make direct internet connections that a user would want to route through a VPN.
	 */
	private val packageBlocklist = setOf(
		"com.android.camera",
		"com.android.camera2",
		"com.google.android.deskclock",
		"com.android.deskclock",
		"com.google.android.calculator",
		"com.android.calculator2",
		"com.google.android.calendar",
		"com.android.providers.calendar",
		"com.android.contacts",
		"com.google.android.contacts",
		"com.android.gallery3d",
		"com.google.android.apps.photos",
		"com.android.settings",
		"com.android.systemui",
		"com.android.stk",
		"com.android.stk2",
		"com.google.android.apps.safetyhub",
	)

	private val applicationFilterPredicate: (ApplicationInfo, PackageManager) -> Boolean = { appInfo, pm ->
		pm.hasInternetPermission(appInfo.packageName) &&
			!isSelfApplication(appInfo.packageName) &&
			!packageBlocklist.contains(appInfo.packageName)
	}

	suspend fun getInstalledApp(packageManager: PackageManager, splitTunnelingRepository: SplitTunnelingRepository): Pair<List<AppInfo>, List<AppInfo>> {
		val savedAppsInfo = splitTunnelingRepository.getAppInfoList().associateBy { it.packageName }

		val normalApps = mutableListOf<AppInfo>()
		val systemApps = mutableListOf<AppInfo>()

		val installedApps = packageManager.getInstalledApplications(PackageManager.GET_META_DATA)
			.filter { applicationFilterPredicate(it, packageManager) }
			.distinctBy {
				it.packageName
			}

		for (appInfo in installedApps) {
			val name = appInfo.loadLabel(packageManager).toString()
			val icon = appInfo.icon

			val app = AppInfo(
				name = name,
				packageName = appInfo.packageName,
				icon = icon,
				passThroughVpn = savedAppsInfo[appInfo.packageName]?.passThroughVpn ?: true,
			)

			if (appInfo.flags and ApplicationInfo.FLAG_SYSTEM != 0 || appInfo.flags and ApplicationInfo.FLAG_UPDATED_SYSTEM_APP != 0) {
				if (packageManager.isLaunchable(appInfo.packageName)) systemApps.add(app)
			} else {
				normalApps.add(app)
			}
		}

		splitTunnelingRepository.saveAppInfoList(systemApps + normalApps)

		val sortedSystemApps = systemApps.sortedBy { app -> app.name }
		val sortedNormalApps = normalApps.sortedBy { app -> app.name }
		return sortedSystemApps to sortedNormalApps
	}

	fun filterApps(query: String, systemApps: List<AppInfo>, normalApps: List<AppInfo>): Pair<List<AppInfo>, List<AppInfo>> {
		val queryFilteredSystemApps = systemApps.filter { app -> app.name.contains(query, ignoreCase = true) }
		val queryFilteredNormalApps = normalApps.filter { app -> app.name.contains(query, ignoreCase = true) }
		return queryFilteredSystemApps to queryFilteredNormalApps
	}

	fun filterDirectApps(appliedFilter: AppFilter, systemApps: List<AppInfo>, normalApps: List<AppInfo>): Pair<List<AppInfo>, List<AppInfo>> {
		val isAlreadySelected = appliedFilter == AppFilter.Direct
		val filteredSystemApps = if (isAlreadySelected) systemApps else systemApps.filterAllPassThroughValue(false)
		val filteredNormalApps = if (isAlreadySelected) normalApps else normalApps.filterAllPassThroughValue(false)
		return filteredSystemApps to filteredNormalApps
	}

	private fun PackageManager.hasInternetPermission(packageName: String): Boolean {
		return PackageManager.PERMISSION_GRANTED ==
			checkPermission(Manifest.permission.INTERNET, packageName)
	}

	private fun PackageManager.isLaunchable(packageName: String): Boolean {
		return getLaunchIntentForPackage(packageName) != null ||
			getLeanbackLaunchIntentForPackage(packageName) != null
	}

	private fun isSelfApplication(packageName: String): Boolean {
		return packageName == BuildConfig.APPLICATION_ID
	}
}

fun List<AppInfo>.updatePassThroughValue(packageName: String) = map { appInfo -> if (appInfo.packageName == packageName) appInfo.copy(passThroughVpn = !appInfo.passThroughVpn) else appInfo }

fun List<AppInfo>.filterAllPassThroughValue(passThroughVpn: Boolean) = filter { appInfo -> appInfo.passThroughVpn == passThroughVpn }

fun List<AppInfo>.totalAppCounts(passThroughVpn: Boolean) = filter { app -> app.passThroughVpn == passThroughVpn }.size
