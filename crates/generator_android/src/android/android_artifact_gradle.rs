fn android_theme(design_config: &DesignConfig) -> String {
    android_theme_module(design_config)
}

fn android_responsive() -> String {
    r#"package dev.dowe.generated

object DoweResponsiveModule {
    const val generated = true
}
"#
    .to_string()
}

fn settings_gradle() -> String {
    r#"pluginManagement { repositories { google(); mavenCentral(); gradlePluginPortal() } }
dependencyResolutionManagement { repositoriesMode.set(RepositoriesMode.FAIL_ON_PROJECT_REPOS); repositories { google(); mavenCentral() } }
rootProject.name = "DoweGeneratedAndroid"
include(":app")
"#
    .to_string()
}

fn root_gradle() -> String {
    r#"plugins {
    id("com.android.application") version "8.13.1" apply false
    id("org.jetbrains.kotlin.android") version "2.2.21" apply false
    id("org.jetbrains.kotlin.plugin.compose") version "2.2.21" apply false
}
"#
    .to_string()
}

fn gradle_properties() -> String {
    r#"android.useAndroidX=true
kotlin.jvm.target.validation.mode=warning
org.gradle.jvmargs=-Xmx2048m -XX:MaxMetaspaceSize=1024m -Dfile.encoding=UTF-8
kotlin.daemon.jvmargs=-Xmx8192m
org.gradle.workers.max=2
"#
    .to_string()
}

fn app_gradle(app_bundle: &str) -> String {
    format!(
        r#"plugins {{
    id("com.android.application")
    id("org.jetbrains.kotlin.android")
    id("org.jetbrains.kotlin.plugin.compose")
}}

android {{
    namespace = "dev.dowe.generated"
    compileSdk = 36

    signingConfigs {{
        create("release") {{
            storeFile = file(System.getenv("DOWE_ANDROID_KEYSTORE") ?: "")
            storePassword = System.getenv("DOWE_ANDROID_KEYSTORE_PASSWORD")
            keyAlias = System.getenv("DOWE_ANDROID_KEY_ALIAS")
            keyPassword = System.getenv("DOWE_ANDROID_KEY_PASSWORD")
        }}
    }}

    defaultConfig {{
        applicationId = "{}"
        minSdk = 26
        targetSdk = 36
        versionCode = (System.getenv("DOWE_APP_BUILD_NUMBER") ?: "1").toInt()
        versionName = System.getenv("DOWE_APP_VERSION") ?: "0.1.0"
    }}

    buildTypes {{
        getByName("release") {{
            signingConfig = signingConfigs.getByName("release")
            isMinifyEnabled = false
        }}
    }}

    compileOptions {{
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }}
}}

kotlin {{
    compilerOptions {{
        jvmTarget.set(org.jetbrains.kotlin.gradle.dsl.JvmTarget.JVM_17)
    }}
}}

dependencies {{
    implementation("androidx.activity:activity-compose:1.11.0")
    implementation(platform("androidx.compose:compose-bom:2026.06.01"))
    implementation("androidx.compose.material3:material3")
    implementation("androidx.compose.ui:ui")
}}
"#,
        escape_kotlin(app_bundle)
    )
}

