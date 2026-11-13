plugins {
    alias(libs.plugins.android.application)
    alias(libs.plugins.kotlin.compose)
}

android {
    namespace = "ai.decentralized.app"
    compileSdk = 36

    defaultConfig {
        applicationId = "ai.decentralized.app"
        minSdk = 26
        targetSdk = 36
        versionCode = 1
        versionName = "0.1.0"
        ndk {
            abiFilters += listOf("arm64-v8a", "armeabi-v7a", "x86_64")
        }
    }

    buildFeatures {
        compose = true
    }
}

val repositoryRoot = rootProject.projectDir.parentFile.parentFile
val rustAndroidOutput = file("src/main/jniLibs")
val buildRustAndroid by tasks.registering(Exec::class) {
    workingDir = repositoryRoot
    commandLine(
        "cargo", "ndk",
        "-t", "arm64-v8a",
        "-t", "armeabi-v7a",
        "-t", "x86_64",
        "-o", rustAndroidOutput.absolutePath,
        "build", "--release", "-p", "decentralized-ai-android-bridge",
    )
    inputs.files(fileTree(repositoryRoot) {
        include(
            "Cargo.toml",
            "Cargo.lock",
            "core/task-protocol/**",
            "core/p2p/**",
            "core/android-bridge/**",
        )
    })
    outputs.dir(rustAndroidOutput)
}

tasks.named("preBuild").configure {
    dependsOn(buildRustAndroid)
}

dependencies {
    implementation(platform(libs.androidx.compose.bom))
    implementation(libs.androidx.activity.compose)
    implementation(libs.androidx.compose.ui)
    implementation(libs.androidx.compose.ui.tooling.preview)
    implementation(libs.androidx.compose.material3)
    implementation(libs.androidx.work.runtime)
    debugImplementation(libs.androidx.compose.ui.tooling)
}
