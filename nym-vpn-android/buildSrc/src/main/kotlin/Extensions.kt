import org.gradle.api.Project
import java.io.File
import java.util.Properties

fun Project.getLocalProperty(key: String, file: String = "local.properties"): String? {
	val properties = java.util.Properties()
	val localProperties = File(file)
	if (localProperties.isFile) {
		java.io.InputStreamReader(java.io.FileInputStream(localProperties), Charsets.UTF_8)
			.use { reader ->
				properties.load(reader)
			}
	} else return null
	return properties.getProperty(key)
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


fun Project.getSigningProperties(): Properties {
	return Properties().apply {
		// created local file for signing details
		try {
			load(file("signing.properties").reader())
		} catch (_: Exception) {
			load(file("signing_template.properties").reader())
		}
	}
}

fun Project.getStoreFile(): File {
	return file(
		System.getenv()
			.getOrDefault(
				Constants.KEY_STORE_PATH_VAR,
				getSigningProperties().getProperty(Constants.KEY_STORE_PATH_VAR),
			),
	)
}

fun Project.getSigningProperty(property: String): String {
	// try to get secrets from env first for pipeline build, then properties file for local
	return System.getenv()
		.getOrDefault(
			property,
			getSigningProperties().getProperty(property),
		)
}


fun Project.isBundleBuild(): Boolean {
	return gradle.startParameter.taskNames.find { it.lowercase().contains("bundle") } != null
}

fun Project.languageList(): List<String> {
	return listOf(
		"ar",      // Arabic
		"bn-rBD",  // Bengali (Bangladesh)
		"de",      // German
		"en",      // English (Default)
		"es",      // Spanish
		"fa",      // Persian
		"fr",      // French
		"hi",      // Hindi
		"pt-rBR",  // Portuguese (Brazil)
		"ru",      // Russian
		"tr",      // Turkish
		"uk",      // Ukrainian
		"vi-rVN",  // Vietnamese
		"zh-rCN"   // Chinese (Simplified)
	).sorted()
}
