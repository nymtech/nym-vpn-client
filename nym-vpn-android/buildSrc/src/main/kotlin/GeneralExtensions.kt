import org.gradle.api.JavaVersion
import org.gradle.api.Project
import java.io.File
import java.io.FileOutputStream
import java.net.URL
import java.util.Properties

fun Project.getJavaVersion() : JavaVersion {
	return JavaVersion.VERSION_17
}

fun Project.getJavaTarget() : String {
	return Constants.JVM_TARGET
}

fun Project.getAllowedLicenses() : List<String> {
	return listOf("MIT", "Apache-2.0", "BSD-3-Clause")
}

fun Project.getJniArchs() : List<String> {
	return listOf("arm64-v8a", "armeabi-v7a, x86, x86_64")
}

fun Project.removeJniLibsFile(fileName : String) {
	fun cleanSharedLibs() {
		getJniArchs().forEach {
			delete("${projectDir.path}/src/main/jniLibs/$it/$fileName")
		}
	}
}

fun Project.isBundleBuild() : Boolean {
	return gradle.startParameter.taskNames.find { it.lowercase().contains("bundle") } != null
}

fun Project.getBuildTaskName(): String {
	val taskRequestsStr = gradle.startParameter.taskRequests[0].toString()
	return taskRequestsStr.also {
		project.logger.lifecycle("Build task: $it")
	}
}

fun getLocalProperty(key: String, file: String = "local.properties"): String? {
	val properties = Properties()
	val localProperties = File(file)
	if (localProperties.isFile) {
		java.io.InputStreamReader(java.io.FileInputStream(localProperties), Charsets.UTF_8)
			.use { reader ->
				properties.load(reader)
			}
	} else return null
	return properties.getProperty(key)
}

fun Project.getSupportlanguages(): List<String> {
	return fileTree("../app/src/main/res") { include("**/strings.xml") }
		.asSequence()
		.map { stringFile -> stringFile.parentFile.name }
		.map { valuesFolderName -> valuesFolderName.replace("values-", "") }
		.filter { valuesFolderName -> valuesFolderName != "values" }
		.map { languageCode -> languageCode.replace("-r", "_") }
		.distinct()
		.sorted()
		.toList() + "en"
}
