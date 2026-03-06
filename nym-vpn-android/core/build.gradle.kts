import org.gradle.kotlin.dsl.support.listFilesOrdered

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

// 1. Define chained providers at the script level (Configuration time)
val ndkPathProvider: Provider<String> = providers.environmentVariable("ANDROID_NDK_HOME")
	.orElse(providers.gradleProperty("android.ndkDirectory"))
	.orElse(androidComponents.sdkComponents.ndkDirectory.map { it.asFile.absolutePath })

val releaseBuildProvider: Provider<Boolean> = providers.gradleProperty("releaseBuild")
	.map { it == "true" }
	.orElse(false)

val skipBuildProvider: Provider<Boolean> = providers.gradleProperty(Constants.BUILD_LIB_TASK)
	.map { it == "false" }
	.orElse(false)

tasks.register<Exec>(Constants.BUILD_LIB_TASK) {
	// 2. Assign to local variables to prevent capturing the `Build_gradle` script instance inside lambdas
	val localNdkPathProvider = ndkPathProvider
	val localReleaseBuildProvider = releaseBuildProvider
	val localSkipBuildProvider = skipBuildProvider
	val coreDirPath = layout.projectDirectory.dir("../../nym-vpn-core").asFile.absolutePath

	onlyIf { !localSkipBuildProvider.get() }

	commandLine("make", "-C", coreDirPath, "-f", "Android.mk")

	doFirst {
		val ndkPath = localNdkPathProvider.orNull
			?: throw Exception("NDK is not installed. Pass -Pandroid.ndkDirectory or set ANDROID_NDK_HOME.")

		val ndkHome = File(ndkPath)
		val ndkToolchain = ndkHome.resolve("toolchains/llvm/prebuilt").listFilesOrdered().lastOrNull()?.resolve("bin")

		if (ndkToolchain == null || !ndkToolchain.exists()) {
			throw Exception("Cannot determine NDK toolchain bin directory in: ${ndkHome.absolutePath}")
		}

		environment("RELEASE", localReleaseBuildProvider.get().toString())
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
