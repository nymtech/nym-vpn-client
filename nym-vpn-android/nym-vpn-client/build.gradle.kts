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

fun cleanSharedLibs() {
	val jniArch = listOf("arm64-v8a", "armeabi-v7a, x86, x86_64")
	jniArch.forEach {
		delete("${projectDir.path}/src/main/jniLibs/$it/libnym_vpn_lib.so")
	}
}

open class CustomBuildExtension {
	var libVersion: String = ""
}

val buildExtension by extra(CustomBuildExtension())

afterEvaluate {
	if (project.hasProperty("libVersion")) {
		buildExtension.libVersion = project.property("libVersion") as String
	}
}

tasks.named<Delete>("clean") {
	cleanSharedLibs()
}

tasks.register(Constants.BUILD_LIB_TASK) {
	with(buildExtension.libVersion) {
		when {
			isEmpty() -> {
				println("Skipping shared object libraries, assuming already built")
				return@register
			}
			equals("source") -> {
				println("Building shared object libraries from source")
				cleanSharedLibs()
				val ndkPath = android.sdkDirectory.resolve("ndk").listFilesOrdered().lastOrNull()?.path ?: System.getenv("ANDROID_NDK_HOME")
				exec {
					commandLine("echo", "NDK HOME: $ndkPath")
					val script = "${projectDir.path}/src/main/scripts/build-libs.sh"
					commandLine("bash").args(script, ndkPath)
				}
			}
			else -> {
				println("Retrieving share object libraries from release tag")
				cleanSharedLibs()
				//TODO download required shared libs and move them to proper dirs
			}
		}
	}
}


