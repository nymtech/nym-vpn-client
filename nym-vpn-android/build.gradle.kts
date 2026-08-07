// Top-level build file where you can add configuration options common to all sub-projects/modules.
plugins {
	alias(libs.plugins.compose.compiler) apply false
	alias(libs.plugins.androidApplication) apply false
	alias(libs.plugins.hilt.android) apply false
	alias(libs.plugins.ksp) apply false
	alias(libs.plugins.android.library) apply false
	alias(libs.plugins.kotlinxSerialization) apply false
	alias(libs.plugins.gross) apply false
	alias(libs.plugins.licensee) apply false
	alias(libs.plugins.ktlint)
	alias(libs.plugins.detekt)
}

subprojects {
	afterEvaluate {
		tasks.matching { it.name.contains("ArtProfile") }.configureEach {
			enabled = false
		}
	}

	apply {
		plugin(rootProject.libs.plugins.detekt.get().pluginId)
		plugin(rootProject.libs.plugins.ktlint.get().pluginId)
	}

	ktlint {
		debug.set(false)
		verbose.set(true)
		android.set(true)
		outputToConsole.set(true)
		ignoreFailures.set(false)
		enableExperimentalRules.set(true)
		filter {
			exclude("**/generated/**")
			exclude("**/nym_vpn_lib/**")
			exclude("**/nym_vpn_lib_types/**")
			exclude("**/tun_provider/**")
			exclude("**/nym_bridges_types/**")
			include("**/kotlin/**")
		}
	}

	detekt {
		source.setFrom(files("src/main/java", "src/main/kotlin"))
		config.setFrom(rootProject.files("config/detekt.yml"))
		buildUponDefaultConfig = true
	}
	tasks.withType<io.gitlab.arturbosch.detekt.Detekt>().configureEach {
		exclude("**/nym_vpn_lib/**", "**/nym_vpn_lib_types/**", "**/tun_provider/**", "**/nym_bridges_types/**")
	}
}
