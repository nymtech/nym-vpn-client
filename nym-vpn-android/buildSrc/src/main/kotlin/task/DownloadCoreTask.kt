package task

import org.gradle.api.DefaultTask
import org.gradle.api.tasks.Input
import org.gradle.api.tasks.TaskAction
import org.gradle.internal.impldep.org.apache.commons.compress.archivers.tar.TarArchiveEntry
import org.gradle.internal.impldep.org.apache.commons.compress.archivers.tar.TarArchiveInputStream
import org.gradle.internal.impldep.org.apache.commons.compress.compressors.gzip.GzipCompressorInputStream
import org.gradle.kotlin.dsl.withGroovyBuilder
import java.io.BufferedOutputStream
import java.io.File
import java.io.FileInputStream
import java.io.FileOutputStream
import java.io.IOException
import java.io.InputStream
import java.io.OutputStream
import java.net.HttpURLConnection
import java.net.URL
import java.nio.file.Files
import java.nio.file.Paths
import java.util.zip.ZipEntry
import java.util.zip.ZipInputStream

open class DownloadCoreTask : DefaultTask() {
	@Input
	var tag: String = ""

	@Input
	var extractPath: String = ""

	@TaskAction
	fun download() {
		println("Downloading shared object library..")
		if(tag.isNotEmpty() && extractPath.isNotEmpty()) {
			val url = "https://github.com/nymtech/nym-vpn-client/releases/download/nym-vpn-core-$tag/nym-vpn-core-${tag}_android_aarch64.tar.gz"
			val zipFilename = url.substring(url.lastIndexOf('/') + 1)
			val downloadDir = "${project.layout.buildDirectory.get().asFile.path}/downloads"
			val jniLibsDir = "${project.projectDir}/src/main/jniLibs/arm64-v8a"

			File(downloadDir).mkdirs()

			val file = File("$downloadDir/$zipFilename")
			file.parentFile.mkdirs()
			file.outputStream().use { out ->
				java.net.URL(url).openStream().use { inStream ->
					inStream.copyTo(out)
				}
			}
			extractTarGz(file.absolutePath, jniLibsDir)
			file.delete()
		}
	}

	fun extractTarGz(sourceFile: String, destinationDir: String) {
		runCatching {
			Files.createDirectories(Paths.get(destinationDir))
			val processBuilder = ProcessBuilder(
				"tar", "-xvzf", sourceFile, "-C", destinationDir, "--strip-components=1"
			)
			processBuilder.directory(File(destinationDir))
			val process = processBuilder.start()

			process.waitFor()

			if (process.exitValue() != 0) {
				println("Failed to extract ${sourceFile} to ${destinationDir}")
				return
			}
			println("Successfully extracted ${sourceFile} to ${destinationDir}")
		}.onFailure {
			it.printStackTrace()
		}
	}
}
