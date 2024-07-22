plugins {
	alias(libs.plugins.android.library)
	alias(libs.plugins.jetbrainsKotlinAndroid)
}

android {
	namespace = "com.zaneschepke.localization_util"
	compileSdk = Constants.COMPILE_SDK

	defaultConfig {
		minSdk = Constants.MIN_SDK

		buildConfigField("String[]", "LANGUAGES", "new String[]{ ${languageList().joinToString(separator = ", ") { "\"$it\"" }} }")

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
	}
	compileOptions {
		sourceCompatibility = Constants.JAVA_VERSION
		targetCompatibility = Constants.JAVA_VERSION
	}
	kotlinOptions {
		jvmTarget = Constants.JVM_TARGET
	}
	buildFeatures {
		buildConfig = true
	}
}

dependencies {

	implementation(libs.androidx.core.ktx)
	implementation(libs.androidx.appcompat)
	implementation(libs.material)
	testImplementation(libs.junit)
	androidTestImplementation(libs.androidx.junit)
	androidTestImplementation(libs.androidx.espresso.core)

	detektPlugins(libs.detekt.rules.compose)
}
