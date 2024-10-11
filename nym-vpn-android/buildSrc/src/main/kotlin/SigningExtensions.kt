import org.gradle.api.Project
import java.io.File
import java.util.Properties

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
