plugins {
	alias(libs.plugins.android.library)
	alias(libs.plugins.kotlinxSerialization)
	id("kotlin-parcelize")
	alias(libs.plugins.ksp)
	alias(libs.plugins.hilt.android)
}

android {
	namespace = "${Constants.NAMESPACE}.${Constants.VPN_LIB_NAME}"
	compileSdk = Constants.COMPILE_SDK
	ndkVersion = Constants.NDK_VERSION

	lint {
		disable.add("UnsafeOptInUsageError")
	}

	defaultConfig {
		minSdk = Constants.MIN_SDK
		testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
		consumerProguardFiles("consumer-rules.pro")

		ndk {
			abiFilters.clear()
			abiFilters.addAll(listOf("arm64-v8a", "armeabi-v7a", "x86_64"))
		}
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
	}

	flavorDimensions += Constants.TYPE
	productFlavors {
		create(Constants.FDROID) {
			dimension = Constants.TYPE
		}
		create(Constants.GENERAL) {
			dimension = Constants.TYPE
		}
		create(Constants.MOCK) {
			dimension = Constants.TYPE
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

	buildFeatures {
		buildConfig = true
	}
}

kotlin {
	compilerOptions {
		jvmTarget.set(org.jetbrains.kotlin.gradle.dsl.JvmTarget.fromTarget(Constants.JVM_TARGET))
		freeCompilerArgs.addAll("-Xstring-concat=inline")
	}
}

dependencies {
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

	implementation(libs.hilt.android)
	ksp(libs.hilt.android.compiler)

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

// 1. Cache-safe providers (No AGP strict NDK validation)
val envNdkProvider = providers.environmentVariable("ANDROID_NDK_HOME")
val propNdkProvider = providers.gradleProperty("android.ndkDirectory")
val sdkDirProvider = androidComponents.sdkComponents.sdkDirectory
val releaseVariants = setOf(Constants.RELEASE, Constants.PRERELEASE, Constants.NIGHTLY)
val isReleaseBuild = gradle.startParameter.taskRequests
	.flatMap { it.args }
	.any { arg -> releaseVariants.any { variant -> arg.contains(variant, ignoreCase = true) } }

val releaseBuildProvider = providers.gradleProperty("releaseBuild")
	.map { it == "true" }
	.orElse(isReleaseBuild)

val skipBuildProvider = providers.gradleProperty(Constants.BUILD_LIB_TASK)
	.map { it == "false" }
	.orElse(false)

tasks.register<Exec>(Constants.BUILD_LIB_TASK) {
	val localEnvNdk = envNdkProvider
	val localPropNdk = propNdkProvider
	val localSdkDir = sdkDirProvider
	val localReleaseBuild = releaseBuildProvider
	val localSkipBuild = skipBuildProvider
	val coreDirPath = layout.projectDirectory.dir("../../nym-vpn-core").asFile.absolutePath

	onlyIf { !localSkipBuild.get() }

	commandLine("make", "-C", coreDirPath, "-f", "Android.mk")

	doFirst {
		// 2. Restore your old logic: Environment Var -> Gradle Property -> SDK Directory fallback
		val ndkHome = localEnvNdk.orNull?.let { File(it) }
			?: localPropNdk.orNull?.let { File(it) }
			?: localSdkDir.get().asFile.resolve("ndk").listFiles()?.sortedArray()?.lastOrNull()

		if (ndkHome == null || !ndkHome.exists()) {
			throw Exception("Cannot determine Android NDK home. Set ANDROID_NDK_HOME or pass -Pandroid.ndkDirectory")
		}

		val ndkToolchain = ndkHome.resolve("toolchains/llvm/prebuilt").listFiles()?.sortedArray()?.lastOrNull()?.resolve("bin")

		if (ndkToolchain == null || !ndkToolchain.exists()) {
			throw Exception("Cannot determine NDK toolchain directory in: ${ndkHome.absolutePath}")
		}

		environment("RELEASE", localReleaseBuild.get().toString())
		environment("ANDROID_NDK_HOME", ndkHome.absolutePath)
		environment("NDK_TOOLCHAIN_DIR", ndkToolchain.absolutePath)
	}
}

tasks.named("preBuild") {
	dependsOn(Constants.BUILD_LIB_TASK)
}

tasks.register<CleanJniLibsTask>("cleanJniLibs")

tasks.named("clean") {
	dependsOn("cleanJniLibs")
}
