plugins {
	alias(libs.plugins.compose.compiler)
	alias(libs.plugins.androidApplication)
	alias(libs.plugins.jetbrainsKotlinAndroid)
	alias(libs.plugins.hilt.android)
	alias(libs.plugins.ksp)
	alias(libs.plugins.licensee)
	alias(libs.plugins.kotlinxSerialization)
	alias(libs.plugins.gross)
	alias(libs.plugins.sentry)
	alias(libs.plugins.grgit)
}

android {
	namespace = "${Constants.NAMESPACE}.${Constants.APP_NAME}"
	compileSdk = Constants.COMPILE_SDK

	androidResources {
		generateLocaleConfig = true
	}

	defaultConfig {
		applicationId = "${Constants.NAMESPACE}.${Constants.APP_NAME}"
		minSdk = Constants.MIN_SDK
		targetSdk = Constants.TARGET_SDK
		versionCode = determineVersionCode()
		versionName = determineVersionName()

		// keep all language resources
		resourceConfigurations.addAll(languageList())

		testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
		vectorDrawables {
			useSupportLibrary = true
		}
		buildConfigField(
			"String",
			Constants.SENTRY_DSN,
			"\"${(System.getenv(Constants.SENTRY_DSN) ?: getLocalProperty("sentry.dsn")) ?: ""}\"",
		)
		buildConfigField("String", "COMMIT_HASH", "\"${grgitService.service.get().grgit.head().id}\"")
		buildConfigField("Boolean", "IS_SANDBOX", "false")
		proguardFile("fdroid-rules.pro")
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
		applicationVariants.all {
			val variant = this
			variant.outputs
				.map { it as com.android.build.gradle.internal.api.BaseVariantOutputImpl }
				.forEach { output ->
					val fullName =
						Constants.APP_NAME +
							"-${variant.flavorName}" +
							"-${variant.buildType.name}" +
							"-${variant.versionName}"
					buildConfigField("String", "APP_NAME", "\"${fullName}\"")
					variant.resValue("string", "fullVersionName", fullName)
					val outputFileName =
						"$fullName.apk"
					output.outputFileName = outputFileName
				}
		}
		release {
			isDebuggable = false
			isMinifyEnabled = true
			isShrinkResources = true
			proguardFiles(
				getDefaultProguardFile("proguard-android-optimize.txt"),
				"proguard-rules.pro",
			)
			signingConfig = signingConfigs.getByName(Constants.RELEASE)
		}
		debug {
			isMinifyEnabled = false
			isShrinkResources = false
			isDebuggable = true
		}

		create(Constants.PRERELEASE) {
			initWith(buildTypes.getByName(Constants.RELEASE))
		}

		create(Constants.NIGHTLY) {
			initWith(buildTypes.getByName(Constants.RELEASE))
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
		create(Constants.SANDBOX) {
			buildConfigField("String", Constants.FLAVOR, "\"${Constants.SANDBOX}\"")
			dimension = Constants.TYPE
		}
		create(Constants.CANARY) {
			buildConfigField("String", Constants.FLAVOR, "\"${Constants.CANARY}\"")
			dimension = Constants.TYPE
		}
	}
	compileOptions {
		isCoreLibraryDesugaringEnabled = true
		sourceCompatibility = Constants.JAVA_VERSION
		targetCompatibility = Constants.JAVA_VERSION
	}

	kotlinOptions {
		jvmTarget = Constants.JVM_TARGET
	}

	kotlin {
		sourceSets {
			all {
				languageSettings.optIn("kotlin.RequiresOptIn")
				languageSettings.optIn("kotlinx.coroutines.ExperimentalCoroutinesApi")
			}
		}
	}

	licensee {
		Constants.allowedLicenses.forEach { allow(it) }
		allowUrl(Constants.ANDROID_TERMS_URL)
		allowUrl(Constants.XZING_LICENSE_URL)
	}

	gross { enableAndroidAssetGeneration.set(true) }

	sentry {
		tracingInstrumentation {
			org.set("nymtech")
			projectName.set("nym-vpn-android")
			autoUploadProguardMapping.set(false)
		}
	}

	buildFeatures {
		compose = true
		buildConfig = true
	}

	packaging {
		resources {
			excludes += "/META-INF/{AL2.0,LGPL2.1}"
		}
		jniLibs.keepDebugSymbols.add("**/*.so")
	}

	if (isBundleBuild()) {
		defaultConfig.ndk.abiFilters("arm64-v8a")
	}
}

dependencies {

	implementation(project(":nym-vpn-client"))
	implementation(project(":logcat-util"))
	implementation(project(":localization-util"))
	coreLibraryDesugaring(libs.com.android.tools.desugar)

	implementation(libs.androidx.core.ktx)
	implementation(libs.androidx.lifecycle.runtime.ktx)
	implementation(libs.androidx.activity.compose)
	implementation(libs.androidx.appcompat)
	implementation(libs.androidx.material.icons.extended)
	implementation(platform(libs.androidx.compose.bom))
	implementation(libs.androidx.ui)
	implementation(libs.androidx.ui.graphics)
	implementation(libs.androidx.ui.tooling.preview)
	implementation(libs.androidx.material3)
	implementation(libs.androidx.window.core.android)

	testImplementation(libs.junit)
	testImplementation(libs.java.client)
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
	implementation(libs.sentry.sentry.opentelemetry.core)

	// navigation
	implementation(libs.androidx.navigation.compose)
	implementation(libs.androidx.hilt.navigation.compose)

	// hilt
	implementation(libs.hilt.android)
	ksp(libs.hilt.android.compiler)

	// storage
	implementation(libs.androidx.datastore.preferences)
	implementation(libs.androidx.security.crypto)

	// splash
	implementation(libs.androidx.core.splashscreen)

	detektPlugins(libs.detekt.rules.compose)

	// moshi/retrofit
	implementation(libs.retrofit)
	implementation(libs.converter.moshi)
	implementation(libs.moshi)
	implementation(libs.moshi.kotlin)
	// warning here https://github.com/square/moshi/discussions/1752
	ksp(libs.moshi.kotlin.codegen)

	// barcode scanning
	implementation(libs.zxing.android.embedded)
}

fun determineVersionCode(): Int {
	return with(getBuildTaskName().lowercase()) {
		when {
			contains(Constants.NIGHTLY) -> Constants.VERSION_CODE + Constants.NIGHTLY_CODE
			contains(Constants.PRERELEASE) -> Constants.VERSION_CODE + Constants.PRERELEASE_CODE
			else -> Constants.VERSION_CODE
		}
	}
}

fun determineVersionName(): String {
	return with(getBuildTaskName().lowercase()) {
		when {
			contains(Constants.NIGHTLY) || contains(Constants.PRERELEASE) ->
				Constants.VERSION_NAME +
					"-${grgitService.service.get().grgit.head().abbreviatedId}"
			else -> Constants.VERSION_NAME
		}
	}
}
