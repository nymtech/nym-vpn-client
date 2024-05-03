import java.util.Properties

var fdroidApkReleasePath = ""
var generateChecksum = false

plugins {
	alias(libs.plugins.androidApplication)
	alias(libs.plugins.jetbrainsKotlinAndroid)
	alias(libs.plugins.hilt.android)
	alias(libs.plugins.ksp)
	alias(libs.plugins.licensee)
	alias(libs.plugins.kotlinxSerialization)
	alias(libs.plugins.gross)
	alias(libs.plugins.sentry)
}

android {
	namespace = "${Constants.NAMESPACE}.${Constants.APP_NAME}"
	compileSdk = Constants.COMPILE_SDK

	defaultConfig {
		applicationId = "${Constants.NAMESPACE}.${Constants.APP_NAME}"
		minSdk = Constants.MIN_SDK
		targetSdk = Constants.TARGET_SDK
		versionCode = Constants.VERSION_CODE
		versionName = Constants.VERSION_NAME

		testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
		vectorDrawables {
			useSupportLibrary = true
		}
		buildConfigField(
			"String",
			Constants.SENTRY_DSN,
			"\"${(System.getenv(Constants.SENTRY_DSN) ?: getLocalProperty("sentry.dsn")) ?: ""}\"",
		)
		buildConfigField("Boolean", "IS_SANDBOX", "false")
		proguardFile("fdroid-rules.pro")
	}

	signingConfigs {
		create(Constants.RELEASE) {
			val properties =
				Properties().apply {
					// created local file for signing details
					try {
						load(file("signing.properties").reader())
					} catch (_: Exception) {
						load(file("signing_template.properties").reader())
					}
				}
			// try to get secrets from env first for pipeline build, then properties file for local
			// build
			storeFile =
				file(
					System.getenv()
						.getOrDefault(
							Constants.KEY_STORE_PATH_VAR,
							properties.getProperty(Constants.KEY_STORE_PATH_VAR),
						),
				)
			storePassword =
				System.getenv()
					.getOrDefault(
						Constants.STORE_PASS_VAR,
						properties.getProperty(Constants.STORE_PASS_VAR),
					)
			keyAlias =
				System.getenv()
					.getOrDefault(
						Constants.KEY_ALIAS_VAR,
						properties.getProperty(Constants.KEY_ALIAS_VAR),
					)
			keyPassword =
				System.getenv()
					.getOrDefault(
						Constants.KEY_PASS_VAR,
						properties.getProperty(Constants.KEY_PASS_VAR),
					)
		}
	}

	buildTypes {
		applicationVariants.all {
			val variant = this
			variant.outputs
				.map { it as com.android.build.gradle.internal.api.BaseVariantOutputImpl }
				.forEach { output ->
					if (variant.flavorName == Constants.FDROID &&
						variant.buildType.name == Constants.RELEASE
					) {
						fdroidApkReleasePath = output.outputFile.path
						generateChecksum = true
					}
					val fullName =
						Constants.APP_NAME +
							"-${variant.flavorName}" +
							"-${variant.buildType.name}" +
							"-${variant.versionName}" +
							"-${output.getFilter(com.android.build.OutputFile.ABI) ?: "universal"}"
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
	}
	flavorDimensions.add(Constants.TYPE)
	productFlavors {
		create(Constants.FDROID) {
			dimension = Constants.TYPE
		}
		create(Constants.GENERAL) {
			dimension = Constants.TYPE
			proguardFile("proguard-rules.pro")
		}
		create(Constants.SANDBOX) {
			buildConfigField("Boolean", "IS_SANDBOX", "true")
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
	composeOptions {
		kotlinCompilerExtensionVersion = Constants.COMPOSE_COMPILER_EXTENSION_VERSION
	}

	packaging {
		resources {
			excludes += "/META-INF/{AL2.0,LGPL2.1}"
		}
	}

	if (isBundleBuild()) {
		defaultConfig.ndk.abiFilters("arm64-v8a", "armeabi-v7a")
	} else {
		splits {
			abi {
				isEnable = true
				reset()
				include("armeabi-v7a", "arm64-v8a")
				isUniversalApk = isReleaseBuild()
			}
		}
	}
}

dependencies {

	implementation(project(":nym_vpn_client"))
	implementation(project(":logcat_helper"))
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
}
