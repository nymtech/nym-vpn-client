package net.nymtech.nymvpn.data.file

import kotlinx.coroutines.CoroutineDispatcher
import kotlinx.coroutines.flow.Flow
import kotlinx.coroutines.flow.MutableStateFlow
import kotlinx.coroutines.flow.filterNotNull
import kotlinx.coroutines.flow.onStart
import kotlinx.coroutines.flow.update
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock
import kotlinx.coroutines.withContext
import kotlinx.serialization.builtins.ListSerializer
import kotlinx.serialization.json.Json
import net.nymtech.nymvpn.data.GatewayRepository
import net.nymtech.nymvpn.data.domain.Gateways
import net.nymtech.vpn.model.NymGateway
import timber.log.Timber
import java.io.File

/**
 * In-memory gateway lists with a per-list JSON file as offline fallback.
 *
 * The files live in their own directory so a gateway refresh never rewrites the
 * shared preferences file, and the lists are decoded once per process on [ioDispatcher]
 * instead of on every preference change in the collector's context.
 */
class FileGatewayRepository(private val directory: File, private val ioDispatcher: CoroutineDispatcher) : GatewayRepository {

	companion object {
		private const val TAG = "gateway-repo"
		const val DIRECTORY_NAME = "gateways"
		private const val ENTRY_FILE = "entry_gateways.json"
		private const val EXIT_FILE = "exit_gateways.json"
		private const val WG_FILE = "wg_gateways.json"
		private val json = Json { ignoreUnknownKeys = true }
		private val listSerializer = ListSerializer(NymGateway.serializer())
	}

	private val state = MutableStateFlow<Gateways?>(null)
	private val loadMutex = Mutex()

	/** One lock per list so the state update and its file write are serialized and never persist out of order. */
	private val fileLocks = listOf(ENTRY_FILE, EXIT_FILE, WG_FILE).associateWith { Mutex() }

	/** Lists whose last file write failed; retried on the next store even if the content is unchanged. Guarded by [fileLocks]. */
	private val pendingWrites = HashSet<String>()

	// StateFlow only emits when the new value differs (equals), so identical refreshes are conflated here.
	override val gatewayFlow: Flow<Gateways> = state.onStart { ensureLoaded() }.filterNotNull()

	override suspend fun setEntryGateways(gateways: List<NymGateway>) {
		store(ENTRY_FILE, gateways) { it.copy(entryGateways = gateways) }
	}

	override suspend fun setExitGateways(gateways: List<NymGateway>) {
		store(EXIT_FILE, gateways) { it.copy(exitGateways = gateways) }
	}

	override suspend fun setWgGateways(gateways: List<NymGateway>) {
		store(WG_FILE, gateways) { it.copy(wgGateways = gateways) }
	}

	/**
	 * Seeds lists that are still empty from a previous storage format. Lists already present are left alone.
	 * Returns true once every imported list is on disk, so the caller can safely drop the legacy copy.
	 */
	suspend fun importLegacy(entry: List<NymGateway>, exit: List<NymGateway>, wg: List<NymGateway>): Boolean {
		ensureLoaded()
		val current = state.value ?: Gateways()
		// Each list is attempted even if an earlier one failed, so a single bad write does not block the others.
		val results = listOf(
			entry.isEmpty() || current.entryGateways.isNotEmpty() || store(ENTRY_FILE, entry) { it.copy(entryGateways = entry) },
			exit.isEmpty() || current.exitGateways.isNotEmpty() || store(EXIT_FILE, exit) { it.copy(exitGateways = exit) },
			wg.isEmpty() || current.wgGateways.isNotEmpty() || store(WG_FILE, wg) { it.copy(wgGateways = wg) },
		)
		return results.all { it }
	}

	/** Updates the in-memory list and persists it. Returns true when the file is up to date afterwards. */
	private suspend fun store(fileName: String, gateways: List<NymGateway>, transform: (Gateways) -> Gateways): Boolean {
		ensureLoaded()
		return fileLocks.getValue(fileName).withLock {
			var changed = false
			state.update { current ->
				val base = current ?: Gateways()
				val next = transform(base)
				changed = next != base
				next
			}
			if (!changed && fileName !in pendingWrites) return true
			val written = withContext(ioDispatcher) { write(fileName, gateways) }
			if (written) pendingWrites.remove(fileName) else pendingWrites.add(fileName)
			written
		}
	}

	private suspend fun ensureLoaded() {
		if (state.value != null) return
		loadMutex.withLock {
			if (state.value != null) return
			val loaded = withContext(ioDispatcher) {
				Gateways(
					entryGateways = read(ENTRY_FILE),
					exitGateways = read(EXIT_FILE),
					wgGateways = read(WG_FILE),
				)
			}
			state.value = loaded
		}
	}

	private fun read(fileName: String): List<NymGateway> {
		val file = File(directory, fileName)
		if (!file.exists()) return emptyList()
		return runCatching { json.decodeFromString(listSerializer, file.readText()) }
			.onFailure { Timber.tag(TAG).w(it, "GatewayFileUnreadable file=%s", fileName) }
			.getOrDefault(emptyList())
	}

	private fun write(fileName: String, gateways: List<NymGateway>): Boolean = runCatching {
		directory.mkdirs()
		val target = File(directory, fileName)
		val temp = File(directory, "$fileName.tmp")
		temp.writeText(json.encodeToString(listSerializer, gateways))
		if (!temp.renameTo(target)) {
			// Some filesystems refuse to rename over an existing file; retry once with the target removed.
			check(target.delete() || !target.exists()) { "could not remove stale ${target.name}" }
			check(temp.renameTo(target)) { "rename failed" }
		}
	}.onFailure { Timber.tag(TAG).w(it, "GatewayFileWriteFailed file=%s", fileName) }.isSuccess
}
