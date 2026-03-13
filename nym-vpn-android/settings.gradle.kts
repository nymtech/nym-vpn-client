import java.io.File

pluginManagement {
	repositories {
		google {
			content {
				includeGroupByRegex("com\\.android.*")
				includeGroupByRegex("com\\.google.*")
				includeGroupByRegex("androidx.*")
			}
		}
		mavenCentral()
		gradlePluginPortal()
	}
}
plugins {
	id("org.gradle.toolchains.foojay-resolver-convention") version "1.0.0"
}
dependencyResolutionManagement {
	repositoriesMode.set(RepositoriesMode.FAIL_ON_PROJECT_REPOS)
	repositories {
		google()
		mavenCentral()
		maven("https://jitpack.io")
		findRustlsPlatformVerifierMaven()?.let { mavenPath ->
			maven {
				url = uri(mavenPath)
				metadataSources.artifact()
			}
		}
	}
}

fun findRustlsPlatformVerifierMaven(): String? {
	val cargoHome = File(System.getProperty("user.home"), ".cargo/registry/src")
	if (!cargoHome.exists()) {
		logger.warn("Cargo registry not found at ${cargoHome.absolutePath}")
		return null
	}
	cargoHome.listFiles()?.forEach { indexDir ->
		indexDir.listFiles()
			?.filter { it.isDirectory && it.name.startsWith("rustls-platform-verifier-android") }
			?.sortedByDescending { it.name }
			?.firstOrNull()
			?.resolve("maven")
			?.takeIf { it.exists() }
			?.let { return it.absolutePath }
	}
	logger.warn("rustls-platform-verifier-android maven repo not found. Run: cargo fetch --manifest-path=../nym-vpn-core/Cargo.toml")
	return null
}

rootProject.name = "NymVPN"

include(":app")
include(":core")
include(":logcatter")
include(":connectivity")
include(":billing")
