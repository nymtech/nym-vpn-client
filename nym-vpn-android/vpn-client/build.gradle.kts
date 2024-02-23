import com.android.build.gradle.internal.tasks.factory.dependsOn
import org.gradle.kotlin.dsl.support.listFilesOrdered

plugins {
    alias(libs.plugins.android.library)
    alias(libs.plugins.jetbrainsKotlinAndroid)
    id("org.jetbrains.kotlin.plugin.serialization")
    id("kotlin-parcelize")
}

android {

    android {
        ndkVersion = sdkDirectory.resolve("ndk").listFilesOrdered().last().name
    }

    project.tasks.preBuild.dependsOn(Constants.BUILD_LIB_TASK)

    namespace = "${Constants.NAMESPACE}.${Constants.VPN_LIB_NAME}"
    compileSdk = 34

    defaultConfig {
        minSdk = 24
        testInstrumentationRunner = "androidx.test.runner.AndroidJUnitRunner"
        consumerProguardFiles("consumer-rules.pro")
        ndk {
            abiFilters += listOf("arm64-v8a", "armeabi-v7a", "x86_64", "x86")
        }
    }

    buildTypes {
        release {
            isMinifyEnabled = false
            proguardFiles(
                getDefaultProguardFile("proguard-android-optimize.txt"),
                "proguard-rules.pro"
            )
        }
        debug {
            isShrinkResources = false
            isMinifyEnabled = false
        }
        create("applicationVariants") {
        }
    }
    compileOptions {
        sourceCompatibility = Constants.JAVA_VERSION
        targetCompatibility = Constants.JAVA_VERSION
    }
    kotlinOptions {
        jvmTarget = Constants.JVM_TARGET
    }
}

dependencies {

    implementation(libs.androidx.core.ktx)
    implementation(libs.jna.v5140)
    implementation(libs.kotlinx.coroutines.core)

    implementation(libs.kotlinx.serialization.json)
    implementation(libs.timber)
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