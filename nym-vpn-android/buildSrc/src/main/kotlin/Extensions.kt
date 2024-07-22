import org.gradle.api.Project
import java.io.File

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

fun Project.isReleaseBuild(): Boolean {
    return gradle.startParameter.taskNames.find { it.lowercase().contains("release") } != null
}

fun Project.isBundleBuild() : Boolean {
    return gradle.startParameter.taskNames.find { it.lowercase().contains("bundle") } != null
}

fun Project.languageList(): List<String> {
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
