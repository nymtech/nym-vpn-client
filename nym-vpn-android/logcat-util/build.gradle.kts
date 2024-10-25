plugins {
	alias(libs.plugins.android.library)
	alias(libs.plugins.jetbrainsKotlinAndroid)
}

android {
	namespace = "${Constants.NAMESPACE_ROOT}.logcatutil"
	compileSdk = Constants.COMPILE_SDK

	defaultConfig {
		minSdk = Constants.MIN_SDK

		testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
		consumerProguardFiles("consumer-rules.pro")
	}

	buildTypes {
		debug {
			isMinifyEnabled = false
		}
		release {
			isMinifyEnabled = true
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
		flavorDimensions.add(Constants.TYPE)
	}
	compileOptions {
		isCoreLibraryDesugaringEnabled = true
		sourceCompatibility = getJavaVersion()
		targetCompatibility = getJavaVersion()
	}
	kotlinOptions {
		jvmTarget = getJavaTarget()
	}
}

dependencies {
	coreLibraryDesugaring(libs.com.android.tools.desugar)

	implementation(libs.androidx.core.ktx)
	implementation(libs.material)
	testImplementation(libs.junit)
	androidTestImplementation(libs.androidx.junit)
	androidTestImplementation(libs.androidx.espresso.core)

	implementation(libs.timber)

	detektPlugins(libs.detekt.rules.compose)
}
