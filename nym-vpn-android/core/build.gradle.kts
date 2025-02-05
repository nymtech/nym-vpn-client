import com.android.build.gradle.internal.tasks.factory.dependsOn
import org.gradle.kotlin.dsl.support.listFilesOrdered

plugins {
	alias(libs.plugins.android.library)
	alias(libs.plugins.jetbrainsKotlinAndroid)
	alias(libs.plugins.kotlinxSerialization)
	id("kotlin-parcelize")
}

android {

	project.tasks.preBuild.dependsOn(Constants.BUILD_LIB_TASK)

	lint {
		disable.add("UnsafeOptInUsageError")
	}

	android {
		ndkVersion = "25.2.9519653"
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

	implementation(project(":ip-calculator"))
	implementation(project(":connectivity"))
	implementation(libs.androidx.lifecycle.service)
	coreLibraryDesugaring(libs.com.android.tools.desugar)

	implementation(libs.androidx.core.ktx)
	implementation(libs.kotlinx.coroutines.core)
	implementation(libs.androidx.lifecycle.process)

	implementation(libs.kotlinx.serialization)
	implementation(libs.timber)
	implementation(libs.relinker)

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
}

tasks.register<Exec>(Constants.BUILD_LIB_TASK) {
	if (project.hasProperty(Constants.BUILD_LIB_TASK) &&
		project.property(Constants.BUILD_LIB_TASK) == "false"
	) {
		commandLine("echo", "Skipping library build")
		return@register
	}
	// prefer system for reproducible builds
	val ndkPath = System.getenv("ANDROID_NDK_HOME") ?: android.sdkDirectory.resolve("ndk").listFilesOrdered().lastOrNull()?.path
	commandLine("echo", "NDK HOME: $ndkPath")
	val script = "${projectDir.path}/src/main/scripts/build-libs.sh"
	// TODO find a better way to limit builds
	if (file("${projectDir.path}/src/main/jniLibs/arm64-v8a/libnym_vpn_lib.so").exists() &&
		file("${projectDir.path}/src/main/jniLibs/arm64-v8a/libwg.so").exists()
	) {
		commandLine("echo", "Library already compiled")
	} else {
		commandLine("bash").args(script, ndkPath)
	}
}
