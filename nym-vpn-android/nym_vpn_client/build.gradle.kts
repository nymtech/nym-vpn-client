import com.android.build.gradle.internal.tasks.factory.dependsOn
import org.gradle.kotlin.dsl.support.listFilesOrdered

plugins {
    alias(libs.plugins.android.library)
    alias(libs.plugins.jetbrainsKotlinAndroid)
    alias(libs.plugins.kotlinxSerialization)
    id("kotlin-parcelize")
}

android {

    android {
        ndkVersion = sdkDirectory.resolve("ndk").listFilesOrdered().last().name
    }

    project.tasks.preBuild.dependsOn(Constants.BUILD_LIB_TASK)

    namespace = "${Constants.NAMESPACE}.${Constants.VPN_LIB_NAME}"
    compileSdk = Constants.COMPILE_SDK

    defaultConfig {
        minSdk = Constants.MIN_SDK
        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
        consumerProguardFiles("consumer-rules.pro")
        ndk {
            abiFilters += listOf("arm64-v8a", "armeabi-v7a", "x86_64", "x86")
        }
        //TODO change this later to sandbox and mainnet switch depending on build
        buildConfigField("String", "API_URL", "\"${Constants.SANDBOX_API_URL}\"")
        buildConfigField("String", "EXPLORER_URL", "\"${Constants.SANDBOX_EXPLORER_URL}\"")
    }

    buildTypes {
        release {
            isMinifyEnabled = true
            proguardFiles(
                getDefaultProguardFile("proguard-android-optimize.txt"),
                "proguard-rules.pro"
            )
        }
        debug {
            isShrinkResources = false
            isMinifyEnabled = false
        }
        flavorDimensions.add(Constants.TYPE)
        productFlavors {
            create(Constants.FDROID) {
                dimension = Constants.TYPE
            }
            create(Constants.GENERAL) {
                dimension = Constants.TYPE
            }
        }
    }
    compileOptions {
        isCoreLibraryDesugaringEnabled = true
        sourceCompatibility = Constants.JAVA_VERSION
        targetCompatibility = Constants.JAVA_VERSION
    }
    kotlinOptions {
        jvmTarget = Constants.JVM_TARGET
        //R8 kotlinx.serialization
        freeCompilerArgs = listOf(
            "-Xstring-concat=inline"
        )
    }
    buildFeatures {
        buildConfig = true
    }
}

dependencies {
    implementation(project(":logcat_helper"))
    coreLibraryDesugaring(libs.com.android.tools.desugar)

    implementation(libs.androidx.core.ktx)
    implementation(libs.kotlinx.coroutines.core)

    implementation(libs.kotlinx.serialization)
    implementation(libs.timber)
    implementation(libs.jna)

    testImplementation(libs.junit)
    androidTestImplementation(libs.androidx.junit)
    androidTestImplementation(libs.androidx.espresso.core)
    androidTestImplementation(platform(libs.androidx.compose.bom))
    androidTestImplementation(libs.androidx.ui.test.junit4)
}


tasks.register<Exec>(Constants.BUILD_LIB_TASK) {
    val ndkPath = android.sdkDirectory.resolve("ndk").listFilesOrdered().last().path ?: System.getenv("ANDROID_NDK_HOME")
    commandLine("echo", "NDK HOME: $ndkPath")
    val script = "${projectDir.path}/src/main/scripts/build-libs.sh"
    //TODO find a better way to limit builds
    if(file("${projectDir.path}/src/main/jniLibs/arm64-v8a/libnym_vpn_lib.so").exists() &&
        file("${projectDir.path}/src/main/jniLibs/arm64-v8a/libwg.so").exists()) {
        commandLine("echo", "Libs already compiled")
    } else commandLine("bash").args(script, ndkPath)
}

