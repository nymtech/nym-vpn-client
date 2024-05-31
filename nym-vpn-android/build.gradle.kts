// Top-level build file where you can add configuration options common to all sub-projects/modules.
plugins {
	alias(libs.plugins.androidApplication) apply false
	alias(libs.plugins.jetbrainsKotlinAndroid) apply false
	alias(libs.plugins.hilt.android) apply false
	alias(libs.plugins.ksp) apply false
	alias(libs.plugins.android.library) apply false
	alias(libs.plugins.kotlinxSerialization) apply false
	alias(libs.plugins.gross) apply false
	alias(libs.plugins.ktlint)
	alias(libs.plugins.detekt)
}

subprojects {

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
			exclude("**/tun_provider/**")
			include("**/kotlin/**")
		}
	}

	detekt {
		source.setFrom(files("src/main/java", "src/main/kotlin"))
		config.setFrom(rootProject.files("config/detekt.yml"))
		buildUponDefaultConfig = true
	}
	tasks.withType<io.gitlab.arturbosch.detekt.Detekt>().configureEach {
		exclude("**/nym_vpn_lib/**", "**/tun_provider/**")
	}
}
