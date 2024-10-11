import com.android.build.gradle.internal.tasks.factory.dependsOn
import org.gradle.kotlin.dsl.support.listFilesOrdered
import task.DownloadCoreTask

plugins {
	alias(libs.plugins.android.library)
	alias(libs.plugins.jetbrainsKotlinAndroid)
	alias(libs.plugins.kotlinxSerialization)
	id("kotlin-parcelize")
}

android {
	namespace = "${Constants.NAMESPACE_ROOT}.${Constants.VPN_LIB_NAME}"
	compileSdk = Constants.COMPILE_SDK

	if (project.hasProperty(Constants.CORE_BUILD_PROP)) {
		val coreBuild = project.property(Constants.CORE_BUILD_PROP) as String
		if (Regex("^v\\d+\\.\\d+\\.\\d+\$").matches(coreBuild)) {
			tasks.register<DownloadCoreTask>(Constants.DOWNLOAD_LIB_TASK) {
				tag = coreBuild
				extractPath = "/nym-vpn-client/src/main/jniLibs/arm64-v8a"
			}
			tasks.preBuild.dependsOn(Constants.DOWNLOAD_LIB_TASK)
		} else {
			tasks.preBuild.dependsOn(Constants.BUILD_SOURCE_TASK)
		}
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
		sourceCompatibility = getJavaVersion()
		targetCompatibility = getJavaVersion()
	}
	kotlinOptions {
		jvmTarget = getJavaTarget()
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

	implementation(project(":localization-util"))
	implementation(project(":ip-calculator"))
	implementation(libs.androidx.lifecycle.service)
	coreLibraryDesugaring(libs.com.android.tools.desugar)

	implementation(libs.androidx.core.ktx)
	implementation(libs.kotlinx.coroutines.core)

	implementation(libs.kotlinx.serialization)
	implementation(libs.timber)
	implementation(libs.jna)
	implementation(libs.relinker)

	testImplementation(libs.junit)
	androidTestImplementation(libs.androidx.junit)
	androidTestImplementation(libs.androidx.espresso.core)
	androidTestImplementation(platform(libs.androidx.compose.bom))
	androidTestImplementation(libs.androidx.ui.test.junit4)

	detektPlugins(libs.detekt.rules.compose)
}

tasks.named<Delete>(Constants.CLEAN_TASK) {
	removeJniLibsFile(Constants.NYM_SHARED_LIB)
	removeJniLibsFile(Constants.WG_SHARED_LIB)
}

tasks.register<Exec>(Constants.BUILD_SOURCE_TASK) {
	dependsOn(Constants.CLEAN_TASK)
	val ndkPath = android.sdkDirectory.resolve("ndk").listFilesOrdered().lastOrNull()?.path ?: System.getenv("ANDROID_NDK_HOME")
	commandLine("echo", "NDK HOME: $ndkPath")
	val script = "${projectDir.path}/src/main/scripts/build-libs.sh"
	commandLine("bash").args(script, ndkPath)
}
