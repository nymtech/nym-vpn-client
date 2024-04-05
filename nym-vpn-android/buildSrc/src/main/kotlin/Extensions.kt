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
    return (gradle.startParameter.taskNames.size > 0 &&
            gradle.startParameter.taskNames[0].contains(
                "Release",
            ))
}