import java.util.Properties

plugins {
    alias(libs.plugins.androidApplication)
    alias(libs.plugins.jetbrainsKotlinAndroid)
    alias(libs.plugins.hilt.android)
    alias(libs.plugins.ksp)
    id("org.jetbrains.kotlin.plugin.serialization")
}

android {
    namespace = Constants.APP_ID
    compileSdk = Constants.COMPILE_SDK
    ndkVersion = Constants.NDK_VERSION

    defaultConfig {
        applicationId = Constants.APP_ID
        minSdk = Constants.MIN_SDK
        targetSdk = Constants.TARGET_SDK
        versionCode = Constants.VERSION_CODE
        versionName = Constants.VERSION_NAME

        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
        vectorDrawables {
            useSupportLibrary = true
        }
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
                    val outputFileName =
                        "${Constants.APP_NAME}-${variant.flavorName}-${variant.buildType.name}-${variant.versionName}.apk"
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
            isDebuggable = true
        }
    }
    flavorDimensions.add(Constants.TYPE)
    productFlavors {
        create(Constants.FDROID) {
            dimension = Constants.TYPE
            proguardFile("fdroid-rules.pro")
        }
        create(Constants.GENERAL) {
            dimension = Constants.TYPE
        }
    }
    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }
    kotlinOptions {
        jvmTarget = Constants.JVM_TARGET
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
}

dependencies {

    implementation(project(":vpn-client"))

    implementation(libs.androidx.core.ktx)
    implementation(libs.androidx.lifecycle.runtime.ktx)
    implementation(libs.androidx.activity.compose)
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
    debugImplementation(libs.androidx.ui.tooling)
    debugImplementation(libs.androidx.ui.test.manifest)

    //uniffi
    implementation(libs.jna)

    // util
    implementation(libs.accompanist.systemuicontroller)
    implementation(libs.lifecycle.runtime.compose)
    implementation(libs.kotlinx.serialization.json)
    implementation(libs.androidx.window)

    // logging
    implementation(libs.timber)

    //navigation
    implementation(libs.androidx.navigation.compose)
    implementation(libs.androidx.hilt.navigation.compose)

    // hilt
    implementation(libs.hilt.android)
    ksp(libs.hilt.android.compiler)

    //storage
    implementation(libs.androidx.datastore.preferences)

    //splash
    implementation(libs.androidx.core.splashscreen)

}

