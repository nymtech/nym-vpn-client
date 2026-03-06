package net.nymtech.nymvpn.util

import android.os.Build
import timber.log.Timber

object GraphicsFallback {

	private fun String.norm() = trim().lowercase()

	private data class AffectedDevice(val manufacturer: String? = null, val brand: String? = null, val model: String? = null, val product: String? = null, val sdk: Int? = null)

	private val affectedDevices = listOf(
		// Nokia G22
		AffectedDevice(manufacturer = "hmd", model = "g22", sdk = 34),
		AffectedDevice(manufacturer = "nokia", model = "g22", sdk = 34),
		AffectedDevice(brand = "nokia", model = "g22", sdk = 34),
		// SKYEGG K11_EEA
		AffectedDevice(manufacturer = "skyegg", model = "k11", sdk = 34),
		AffectedDevice(brand = "skyegg", model = "k11", sdk = 34),
		AffectedDevice(product = "k11", model = "k11", sdk = 34),
	)

	private fun matchesDevice(entry: AffectedDevice): Boolean {
		val manOk = entry.manufacturer?.let { Build.MANUFACTURER.norm().contains(it.norm()) } ?: true
		val brandOk = entry.brand?.let { Build.BRAND.norm().contains(it.norm()) } ?: true
		val modelOk = entry.model?.let { Build.MODEL.norm().contains(it.norm()) } ?: true
		val prodOk = entry.product?.let { Build.PRODUCT.norm().contains(it.norm()) } ?: true
		val sdkOk = entry.sdk?.let { Build.VERSION.SDK_INT == it } ?: true
		return manOk && brandOk && modelOk && prodOk && sdkOk
	}

	private fun isAffectedDevice(): Boolean = affectedDevices.any { matchesDevice(it) }

	fun applyIfNeeded() {
		if (isAffectedDevice()) {
			try {
				System.setProperty("debug.hwui.renderer", "opengl")
				System.setProperty("ro.hwui.use_vulkan", "false")
				Timber.d("OpenGL fallback:  ${Build.MANUFACTURER} ${Build.MODEL} (SDK ${Build.VERSION.SDK_INT})")
			} catch (t: Throwable) {
				Timber.d("Failed to set graphics fallback: $t ")
			}
		}
	}
}
