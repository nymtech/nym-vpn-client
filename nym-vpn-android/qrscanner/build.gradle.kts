plugins {
	alias(libs.plugins.android.library)
	alias(libs.plugins.compose.compiler)
}

android {
	namespace = "net.nymtech.qrscanner"
	compileSdk = Constants.COMPILE_SDK

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
		debug {
			isMinifyEnabled = false
		}
		release {
			isMinifyEnabled = true
			vcsInfo.include = false
			proguardFiles(
				getDefaultProguardFile("proguard-android-optimize.txt"),
				"proguard-rules.pro",
			)
		}
		create(Constants.PRERELEASE) {
			initWith(buildTypes.getByName(Constants.RELEASE))
		}
		create(Constants.NIGHTLY) {
			initWith(buildTypes.getByName(Constants.RELEASE))
		}
	}
	flavorDimensions += Constants.TYPE
	buildFeatures {
		compose = true
	}
	compileOptions {
		sourceCompatibility = Constants.JAVA_VERSION
		targetCompatibility = Constants.JAVA_VERSION
	}
}

kotlin {
	compilerOptions {
		jvmTarget.set(org.jetbrains.kotlin.gradle.dsl.JvmTarget.fromTarget(Constants.JVM_TARGET))
	}
}

dependencies {
	implementation(libs.androidx.core.ktx)
	implementation(libs.androidx.activity.compose)
	implementation(libs.androidx.lifecycle.runtime.ktx)
	implementation(libs.lifecycle.runtime.compose)
	implementation(libs.lifecycle.viewmodel.compose)

	// compose
	implementation(platform(libs.androidx.compose.bom))
	implementation(libs.androidx.ui)
	implementation(libs.androidx.ui.graphics)
	implementation(libs.androidx.ui.tooling.preview)
	implementation(libs.androidx.material3)
	implementation(libs.androidx.material.icons.extended)
	debugImplementation(libs.androidx.ui.tooling)

	// camera
	implementation(libs.androidx.camera.core)
	implementation(libs.androidx.camera.camera2)
	implementation(libs.androidx.camera.lifecycle)
	implementation(libs.androidx.camera.view)

	// qr decoding, on-device only
	implementation(libs.zxing.core)

	// logging
	implementation(libs.timber)

	testImplementation(libs.junit)
	testImplementation(libs.kotlinx.coroutines.test)
	androidTestImplementation(libs.androidx.junit)
	androidTestImplementation(libs.androidx.espresso.core)

	detektPlugins(libs.detekt.rules.compose)
}
