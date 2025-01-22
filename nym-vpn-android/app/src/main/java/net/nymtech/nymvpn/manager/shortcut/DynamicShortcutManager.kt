package net.nymtech.nymvpn.manager.shortcut

import android.content.Context
import android.content.Intent
import android.os.Build
import androidx.annotation.RequiresApi
import androidx.core.content.pm.ShortcutInfoCompat
import androidx.core.content.pm.ShortcutManagerCompat
import androidx.core.graphics.drawable.IconCompat
import net.nymtech.nymvpn.R
import net.nymtech.nymvpn.ui.ShortcutActivity

class DynamicShortcutManager(val context: Context) : ShortcutManager {
	override fun addShortcuts() {
		if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.N_MR1) {
			ShortcutManagerCompat.setDynamicShortcuts(context, createShortcuts())
		}
	}

	override fun removeShortcuts() {
		if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.N_MR1) {
			ShortcutManagerCompat.removeDynamicShortcuts(context, createShortcuts().map { it.id })
		}
	}

	@RequiresApi(Build.VERSION_CODES.N_MR1)
	private fun createShortcuts(): List<ShortcutInfoCompat> {
		return listOf(
			buildShortcut(
				context.getString(R.string.five_hop_mixnet),
				context.getString(R.string.five_hop_mixnet),
				context.getString(R.string.five_hop_mixnet),
				intent = Intent(context, ShortcutActivity::class.java).apply {
					action = ShortcutAction.START_MIXNET.name
				},
				shortcutIcon = R.drawable.visibility_off,
			),
			buildShortcut(
				context.getString(R.string.two_hop_title),
				context.getString(R.string.two_hop_title),
				context.getString(R.string.two_hop_title),
				intent = Intent(context, ShortcutActivity::class.java).apply {
					action = ShortcutAction.START_WG.name
				},
				shortcutIcon = R.drawable.speed,
			),
			buildShortcut(
				context.getString(R.string.disconnect),
				context.getString(R.string.disconnect),
				context.getString(R.string.disconnect),
				intent = Intent(context, ShortcutActivity::class.java).apply {
					action = ShortcutAction.STOP.name
				},
				shortcutIcon = R.drawable.stop,
			),
		)
	}

	@RequiresApi(Build.VERSION_CODES.N_MR1)
	private fun buildShortcut(id: String, shortLabel: String, longLabel: String, intent: Intent, shortcutIcon: Int): ShortcutInfoCompat {
		return ShortcutInfoCompat.Builder(context, id)
			.setShortLabel(shortLabel)
			.setLongLabel(longLabel)
			.setIntent(intent)
			.setIcon(IconCompat.createWithResource(context, shortcutIcon))
			.build()
	}
}
