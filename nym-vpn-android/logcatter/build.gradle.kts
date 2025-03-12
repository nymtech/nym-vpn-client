plugins {
	alias(libs.plugins.android.library)
	alias(libs.plugins.jetbrainsKotlinAndroid)
}

android {
	namespace = "net.nymtech.logcatter"
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
		flavorDimensions.add(Constants.TYPE)
	}
	compileOptions {
		isCoreLibraryDesugaringEnabled = true
		sourceCompatibility = Constants.JAVA_VERSION
		targetCompatibility = Constants.JAVA_VERSION
	}
	kotlinOptions {
		jvmTarget = Constants.JVM_TARGET
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

	implementation(libs.androidx.lifecycle.process)
}
