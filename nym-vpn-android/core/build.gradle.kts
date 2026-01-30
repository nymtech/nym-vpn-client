import org.gradle.kotlin.dsl.support.listFilesOrdered

plugins {
	alias(libs.plugins.android.library)
	alias(libs.plugins.jetbrainsKotlinAndroid)
	alias(libs.plugins.kotlinxSerialization)
	id("kotlin-parcelize")
}

android {

	lint {
		disable.add("UnsafeOptInUsageError")
	}

	android {
		ndkVersion = "28.2.13676358"
	}

	namespace = "${Constants.NAMESPACE}.${Constants.VPN_LIB_NAME}"
	compileSdk = Constants.COMPILE_SDK

	defaultConfig {
		minSdk = Constants.MIN_SDK
		testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
		consumerProguardFiles("consumer-rules.pro")
	}

	buildTypes {
		release {
			isMinifyEnabled = true
			proguardFiles(
				getDefaultProguardFile("proguard-android-optimize.txt"),
				"proguard-rules.pro",
			)
		}
		debug {
			isShrinkResources = false
			isMinifyEnabled = false
		}

		create(Constants.PRERELEASE) {
			initWith(buildTypes.getByName(Constants.RELEASE))
		}

		create(Constants.NIGHTLY) {
			initWith(buildTypes.getByName(Constants.RELEASE))
		}

		flavorDimensions.add(Constants.TYPE)
		productFlavors {
			create(Constants.FDROID) {
				dimension = Constants.TYPE
			}
			create(Constants.GENERAL) {
				dimension = Constants.TYPE
			}
		}
	}

	packaging {
		jniLibs.keepDebugSymbols.add("**/*.so")
	}

	compileOptions {
		isCoreLibraryDesugaringEnabled = true
		sourceCompatibility = Constants.JAVA_VERSION
		targetCompatibility = Constants.JAVA_VERSION
	}
	kotlinOptions {
		jvmTarget = Constants.JVM_TARGET
		// R8 kotlinx.serialization
		freeCompilerArgs =
			listOf(
				"-Xstring-concat=inline",
			)
	}
	buildFeatures {
		buildConfig = true
	}
}

dependencies {
	// for allowsIps calculator (future)
	// for monitoring network offline status
	implementation(project(":connectivity"))
	implementation(libs.androidx.lifecycle.service)
	coreLibraryDesugaring(libs.com.android.tools.desugar)

	implementation(libs.androidx.core.ktx)
	implementation(libs.kotlinx.coroutines.core)
	implementation(libs.androidx.lifecycle.process)

	implementation(libs.kotlinx.serialization)
	implementation(libs.timber)
	implementation(libs.relinker)
	implementation(libs.semver4j)

	implementation(libs.jna) {
		artifact {
			type = "aar"
		}
	}

	testImplementation(libs.junit)
	androidTestImplementation(libs.androidx.junit)
	androidTestImplementation(libs.androidx.espresso.core)
	androidTestImplementation(platform(libs.androidx.compose.bom))
	androidTestImplementation(libs.androidx.ui.test.junit4)

	detektPlugins(libs.detekt.rules.compose)

	implementation(libs.androidx.datastore.preferences)
}

// this task builds the native core from source and moves the files to the jniLibs dir
tasks.register<Exec>(Constants.BUILD_LIB_TASK) {
	if (project.hasProperty(Constants.BUILD_LIB_TASK) &&
		project.property(Constants.BUILD_LIB_TASK) == "false"
	) {
		commandLine("echo", "Skipping library build")
		return@register
	}

	// prefer system for reproducible builds
	var ndkHome = System.getenv("ANDROID_NDK_HOME")?.let { File(it) } ?: android.sdkDirectory.resolve("ndk").listFilesOrdered().lastOrNull()
	if (ndkHome == null) {
		throw Exception("Cannot determine Android NDK home")
	}

	var ndkToolchain = ndkHome.resolve("toolchains/llvm/prebuilt").listFilesOrdered().lastOrNull()?.resolve("bin")
	if (ndkToolchain == null) {
		throw Exception("Cannot determine NDK toolchain directory")
	}

	// todo: how to do this properly with gradle?
	val isReleaseBuild = project.gradle.startParameter.taskNames.any { it.lowercase().contains("release") }

	val coreDir = "${projectDir.path}/../../nym-vpn-core"
	commandLine("make", "-C", coreDir, "-f", "Android.mk")
		.environment("RELEASE", isReleaseBuild)
		.environment("ANDROID_NDK_HOME", ndkHome)
		.environment("NDK_TOOLCHAIN_DIR", ndkToolchain)
}

tasks.named("preBuild") {
	dependsOn(Constants.BUILD_LIB_TASK)
}

tasks.register<CleanJniLibsTask>("cleanJniLibs")

tasks.named("clean") {
	dependsOn("cleanJniLibs")
}
