plugins {
	alias(libs.plugins.android.library)
	alias(libs.plugins.jetbrainsKotlinAndroid)
}

android {
	namespace = "${Constants.NAMESPACE_ROOT}.localizationutil"
	compileSdk = Constants.COMPILE_SDK

	defaultConfig {
		minSdk = Constants.MIN_SDK

		buildConfigField("String[]", "LANGUAGES", "new String[]{ ${getSupportlanguages().joinToString(separator = ", ") { "\"$it\"" }} }")

		testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
		consumerProguardFiles("consumer-rules.pro")
	}

	buildTypes {
		release {
			isMinifyEnabled = false
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
	compileOptions {
		sourceCompatibility = getJavaVersion()
		targetCompatibility = getJavaVersion()
	}
	kotlinOptions {
		jvmTarget = getJavaTarget()
	}
	buildFeatures {
		buildConfig = true
	}
}

dependencies {

	implementation(libs.androidx.core.ktx)
	implementation(libs.material)
	testImplementation(libs.junit)
	androidTestImplementation(libs.androidx.junit)
	androidTestImplementation(libs.androidx.espresso.core)

	detektPlugins(libs.detekt.rules.compose)
}
