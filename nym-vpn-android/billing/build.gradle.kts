plugins {
	alias(libs.plugins.android.library)
}

android {
	namespace = "net.nymtech.billing"
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
	implementation(libs.ipaddress)
	implementation(libs.material)

	testImplementation(libs.junit)
	androidTestImplementation(libs.androidx.junit)
	androidTestImplementation(libs.androidx.espresso.core)

	detektPlugins(libs.detekt.rules.compose)

	// logging
	implementation(libs.timber)

	add("generalImplementation", libs.billing.client)
}
