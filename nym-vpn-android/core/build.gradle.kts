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

// this task builds the native core from source and moves the files to the jniLibs dir
tasks.register<Exec>(Constants.BUILD_LIB_TASK) {
	// Gradle 9 safe properties
	val skipBuild = providers.gradleProperty(Constants.BUILD_LIB_TASK).getOrElse("false") == "false"
	val isReleaseBuild = providers.gradleProperty("releaseBuild").getOrElse("false") == "true"
	val coreDirPath = layout.projectDirectory.dir("../../nym-vpn-core").asFile.absolutePath

	onlyIf { !skipBuild }

	commandLine("make", "-C", coreDirPath, "-f", "Android.mk")

	doFirst {
		val ndkPath = providers.environmentVariable("ANDROID_NDK_HOME").orNull
			?: providers.gradleProperty("android.ndkDirectory").orNull
			?: androidComponents.sdkComponents.ndkDirectory.orNull?.asFile?.absolutePath

		if (ndkPath == null) {
			throw Exception("NDK is not installed. Pass -Pandroid.ndkDirectory or set ANDROID_NDK_HOME.")
		}

		val ndkHome = File(ndkPath)
		val ndkToolchain = ndkHome.resolve("toolchains/llvm/prebuilt").listFilesOrdered().lastOrNull()?.resolve("bin")

		if (ndkToolchain == null || !ndkToolchain.exists()) {
			throw Exception("Cannot determine NDK toolchain bin directory in: ${ndkHome.absolutePath}")
		}

		environment("RELEASE", isReleaseBuild.toString())
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
