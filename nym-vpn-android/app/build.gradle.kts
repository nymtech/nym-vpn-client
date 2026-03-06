plugins {
	alias(libs.plugins.compose.compiler)
	alias(libs.plugins.androidApplication)
	alias(libs.plugins.hilt.android)
	alias(libs.plugins.ksp)
	alias(libs.plugins.licensee)
	alias(libs.plugins.kotlinxSerialization)
	alias(libs.plugins.gross)
	alias(libs.plugins.grgit)
}

base {
	archivesName.set("${Constants.APP_NAME}-${determineVersionName()}")
}

android {
	namespace = Constants.APP_ID
	compileSdk = Constants.COMPILE_SDK

	androidResources {
		generateLocaleConfig = true
		localeFilters += languageList()
	}

	dependenciesInfo {
		includeInApk = false
		includeInBundle = false
	}

	defaultConfig {
		applicationId = Constants.APP_ID
		minSdk = Constants.MIN_SDK
		targetSdk = Constants.TARGET_SDK
		versionCode = Constants.VERSION_CODE
		versionName = determineVersionName()

		testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
		vectorDrawables {
			useSupportLibrary = true
		}

		buildConfigField("String[]", "LANGUAGES", "new String[]{ ${languageList().joinToString(separator = ", ") { "\"$it\"" }} }")
		buildConfigField("String", "COMMIT_HASH", "\"${grgitService.service.get().grgit.head().id}\"")
		buildConfigField("Boolean", "IS_PRERELEASE", "false")
		proguardFile("fdroid-rules.pro")

		if (isBundleBuild()) {
			ndk {
				abiFilters += "arm64-v8a"
			}
		}
	}

	signingConfigs {
		create(Constants.RELEASE) {
			storeFile = getStoreFile()
			storePassword = getSigningProperty(Constants.STORE_PASS_VAR)
			keyAlias = getSigningProperty(Constants.KEY_ALIAS_VAR)
			keyPassword = getSigningProperty(Constants.KEY_PASS_VAR)
		}
	}

	buildTypes {
		release {
			isDebuggable = false
			isMinifyEnabled = true
			isShrinkResources = true
			vcsInfo.include = false
			proguardFiles(
				getDefaultProguardFile("proguard-android-optimize.txt"),
				"proguard-rules.pro",
			)
			resValue("string", "provider", "\"${Constants.APP_NAME}.provider\"")
			signingConfig = signingConfigs.getByName(Constants.RELEASE)
		}
		debug {
			isMinifyEnabled = false
			isShrinkResources = false
			isDebuggable = true
			applicationIdSuffix = ".debug"
			versionNameSuffix = "-debug"
			resValue("string", "app_name", "NymVPN - Debug")
			resValue("string", "provider", "\"${Constants.APP_NAME}.provider.debug\"")
		}

		create(Constants.PRERELEASE) {
			initWith(buildTypes.getByName(Constants.RELEASE))
			applicationIdSuffix = ".prerelease"
			versionNameSuffix = "-pre"
			resValue("string", "app_name", "NymVPN - Pre")
			resValue("string", "provider", "\"${Constants.APP_NAME}.provider.pre\"")
			buildConfigField("Boolean", "IS_PRERELEASE", "true")
		}

		create(Constants.NIGHTLY) {
			initWith(buildTypes.getByName(Constants.RELEASE))
			applicationIdSuffix = ".nightly"
			versionNameSuffix = "-nightly"
			resValue("string", "app_name", "NymVPN - Nightly")
			resValue("string", "provider", "\"${Constants.APP_NAME}.provider.nightly\"")
		}
	}

	flavorDimensions.add(Constants.TYPE)
	productFlavors {
		create(Constants.FDROID) {
			dimension = Constants.TYPE
			buildConfigField("String", Constants.FLAVOR, "\"${Constants.FDROID}\"")
		}
		create(Constants.GENERAL) {
			dimension = Constants.TYPE
			proguardFile("proguard-rules.pro")
			buildConfigField("String", Constants.FLAVOR, "\"${Constants.GENERAL}\"")
		}
	}

	compileOptions {
		isCoreLibraryDesugaringEnabled = true
		sourceCompatibility = Constants.JAVA_VERSION
		targetCompatibility = Constants.JAVA_VERSION
	}

	kotlin {
		jvmToolchain(17)
		sourceSets {
			all {
				languageSettings.optIn("kotlin.RequiresOptIn")
				languageSettings.optIn("kotlinx.coroutines.ExperimentalCoroutinesApi")
			}
		}
		compilerOptions {
			jvmTarget.set(org.jetbrains.kotlin.gradle.dsl.JvmTarget.fromTarget(Constants.JVM_TARGET))
		}
	}

	licensee {
		Constants.allowedLicenses.forEach { allow(it) }
		allowUrl(Constants.ANDROID_TERMS_URL)
		allowUrl(Constants.XZING_LICENSE_URL)
	}

	gross { enableAndroidAssetGeneration.set(true) }

	buildFeatures {
		compose = true
		buildConfig = true
		resValues = true
	}

	packaging {
		resources {
			excludes += "/META-INF/{AL2.0,LGPL2.1}"
		}
		jniLibs.keepDebugSymbols.add("**/*.so")
	}
}

androidComponents {
	onVariants { variant ->
		val flavor = variant.flavorName ?: ""
		val type = variant.buildType ?: ""
		val version = variant.outputs.first().versionName.getOrElse("")
		val fullName = "${Constants.APP_NAME}-$flavor-$type-$version"

		variant.buildConfigFields?.put("APP_NAME", com.android.build.api.variant.BuildConfigField("String", "\"$fullName\"", "App Name"))
		variant.resValues.put(variant.makeResValueKey("string", "fullVersionName"), com.android.build.api.variant.ResValue(fullName))
	}
}

dependencies {
	implementation(project(":core"))
	implementation(project(":connectivity"))
	implementation(project(":logcatter"))
	implementation(project(":billing"))
	implementation(libs.androidx.lifecycle.process)
	coreLibraryDesugaring(libs.com.android.tools.desugar)

	implementation(libs.androidx.core.ktx)
	implementation(libs.androidx.lifecycle.runtime.ktx)
	implementation(libs.androidx.activity.compose)

	implementation(libs.androidx.material.icons.extended)
	implementation(platform(libs.androidx.compose.bom))
	implementation(libs.androidx.ui)
	implementation(libs.androidx.ui.graphics)
	implementation(libs.androidx.ui.tooling.preview)
	implementation(libs.androidx.material3)
	implementation(libs.androidx.window.core.android)

	testImplementation(libs.junit)
	androidTestImplementation(libs.androidx.junit)
	androidTestImplementation(libs.androidx.espresso.core)
	androidTestImplementation(platform(libs.androidx.compose.bom))
	androidTestImplementation(libs.androidx.ui.test.junit4)
	androidTestImplementation(libs.androidx.uiautomator)
	debugImplementation(libs.androidx.ui.tooling)
	debugImplementation(libs.androidx.ui.test.manifest)

	// util
	implementation(libs.accompanist.permissions)
	implementation(libs.lifecycle.runtime.compose)
	implementation(libs.kotlinx.serialization)
	implementation(libs.androidx.window)
	implementation(libs.androidx.lifecycle.service)

	// logging
	implementation(libs.timber)

	// navigation
	implementation(libs.androidx.navigation.compose)
	implementation(libs.androidx.hilt.navigation.compose)

	// hilt
	implementation(libs.hilt.android)
	ksp(libs.hilt.android.compiler)

	// storage
	implementation(libs.androidx.datastore.preferences)
	implementation(libs.androidx.security.crypto)

	detektPlugins(libs.detekt.rules.compose)

	// barcode scanning
	implementation(libs.zxing.android.embedded)

	// animations/splash
	implementation(libs.lottie.compose)
	implementation(libs.androidx.core.splashscreen)

	// sentry
	implementation(libs.sentry)

	// biometric
	implementation(libs.biometric)

	// credentials
	implementation(libs.credentials)
}

fun determineVersionName(): String = with(getBuildTaskName().lowercase()) {
	when {
		contains(Constants.NIGHTLY) || contains(Constants.PRERELEASE) ->
			Constants.VERSION_NAME +
				"-${grgitService.service.get().grgit.head().abbreviatedId}"

		else -> Constants.VERSION_NAME
	}
}
