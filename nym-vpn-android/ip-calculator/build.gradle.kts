plugins {
	alias(libs.plugins.android.library)
	alias(libs.plugins.jetbrainsKotlinAndroid)
}

android {
	namespace = "${Constants.NAMESPACE_ROOT}.ipcalculator"
	compileSdk = Constants.COMPILE_SDK

	defaultConfig {
		minSdk = Constants.MIN_SDK

		testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
		consumerProguardFiles("consumer-rules.pro")
	}

	buildTypes {
		release {
			isMinifyEnabled = false
			proguardFiles(getDefaultProguardFile("proguard-android-optimize.txt"), "proguard-rules.pro")
		}
	}
	compileOptions {
		sourceCompatibility = getJavaVersion()
		targetCompatibility = getJavaVersion()
	}
	kotlinOptions {
		jvmTarget = getJavaTarget()
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
}
