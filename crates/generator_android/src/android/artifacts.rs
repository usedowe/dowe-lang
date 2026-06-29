use dowe_components::{
    AccordionItem, AccordionProps, Align, AlertDialogProps, AudioProps, AvatarGroupItem,
    AvatarGroupProps, AvatarProps, BadgeProps, BarProps, BorderWidth, Breakpoint, ButtonSize,
    CarouselProps, CarouselSlide, ChartCommonProps, ChatBoxProps, CheckboxProps, ChipProps,
    CodeToken, CodeTokenKind, ColorFamily, ColorProps, ColorToken, ComboBoxProps, ComboOption,
    CommandEntry, CommandProps, CollapsibleProps, ComponentVariant, CountdownProps, CoverSource,
    CsvColumn, DateProps, DateRangeProps, DesignConfig, DesignTheme, DividerOrientation,
    DividerProps, DragGroup, DragItem, DrawerPosition,
    DrawerProps, DropzoneProps, DropdownProps, ElementProps, EmptyProps, FabAction, FabProps,
    FontConfig, FontFamily, GapSize, GapValue, GridAlignment,
    GridProps, GridTracks, INPUT_HORIZONTAL_PADDING, INPUT_MIN_HEIGHT, INPUT_TEXT_SIZE,
    ImageProps, Justify, LayoutProps, MapMarker, MapProps, MapWaypoint,
    MarqueeProps, ModalProps, NavMenuItem, NavMenuItemProps,
    NavMenuProps, NavigationAction, OverlayCornerPosition, OverlayEntry, OverlayItemProps,
    OverlayPaint, RadioGroupProps,
    RadioOption, RecordProps, ResponsiveValue, RichTextMark,
    RoundedSize, ScaleValue,
    ScaffoldProps, SectionBackground, SelectOption, SideNavIcon, SideNavItem, SideNavItemProps,
    SideNavProps, SideNavSize, SkeletonProps, SizeValue, SliderProps, StyleProps, SvgPath,
    SvgPathFill, SvgViewBox, TabItem, TableColumn, TableColumnAlign, TableProps, TableSize,
    TabsProps, TabsVariant, TextProps, TextSize, TextSpacing, TextWeight, ThemeToggleProps,
    ToastProps, ToggleGroupItem, ToggleGroupProps, ToggleProps, TooltipProps, TranslationCatalog, TypeWriterItem, TypeWriterProps,
    VariantProps, ViewAction, ViewActionKind, ViewAnimation, ViewIcon, ViewNode,
    ViewRequestAction, ViewRoute, ViewSignal, ViewSignalValue, VisibilityCondition,
    collect_route_font_families, compose_tree, node_element_props, text_spacing_em,
    text_typography, translation_resource_name,
};
use std::collections::BTreeSet;
use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AndroidOutput {
    pub files: Vec<AndroidArtifact>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AndroidArtifact {
    pub relative_path: PathBuf,
    pub content: String,
    pub kind: AndroidArtifactKind,
    pub target: &'static str,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AndroidArtifactKind {
    ProjectFile,
    Manifest,
    Entrypoint,
    GeneratedView,
    Routing,
    Layouts,
    Pages,
    Theme,
    Responsive,
    DevEntrypoint,
    Localization,
}

pub fn generate_android(
    routes: &[ViewRoute],
    font_config: &FontConfig,
    design_config: &DesignConfig,
    environment: &[(String, String)],
) -> AndroidOutput {
    generate_android_with_app_and_translations(
        routes,
        font_config,
        design_config,
        environment,
        &TranslationCatalog::default(),
        "Dowe Dev",
        "dev.dowe.generated",
    )
}

pub fn generate_android_with_translations(
    routes: &[ViewRoute],
    font_config: &FontConfig,
    design_config: &DesignConfig,
    environment: &[(String, String)],
    translations: &TranslationCatalog,
) -> AndroidOutput {
    generate_android_with_app_and_translations(
        routes,
        font_config,
        design_config,
        environment,
        translations,
        "Dowe Dev",
        "dev.dowe.generated",
    )
}

pub fn generate_android_with_app_and_translations(
    routes: &[ViewRoute],
    font_config: &FontConfig,
    design_config: &DesignConfig,
    environment: &[(String, String)],
    translations: &TranslationCatalog,
    app_name: &str,
    app_bundle: &str,
) -> AndroidOutput {
    let font_families = font_config.effective_families(&collect_route_font_families(routes));
    let mut files = vec![
        AndroidArtifact {
            relative_path: PathBuf::from("apps/android/settings.gradle.kts"),
            content: settings_gradle(),
            kind: AndroidArtifactKind::ProjectFile,
            target: "android",
        },
        AndroidArtifact {
            relative_path: PathBuf::from("apps/android/build.gradle.kts"),
            content: root_gradle(),
            kind: AndroidArtifactKind::ProjectFile,
            target: "android",
        },
        AndroidArtifact {
            relative_path: PathBuf::from("apps/android/gradle.properties"),
            content: gradle_properties(),
            kind: AndroidArtifactKind::ProjectFile,
            target: "android",
        },
        AndroidArtifact {
            relative_path: PathBuf::from("apps/android/app/build.gradle.kts"),
            content: app_gradle(app_bundle),
            kind: AndroidArtifactKind::ProjectFile,
            target: "android",
        },
        AndroidArtifact {
            relative_path: PathBuf::from("apps/android/app/src/main/AndroidManifest.xml"),
            content: android_manifest(app_name),
            kind: AndroidArtifactKind::Manifest,
            target: "android",
        },
        AndroidArtifact {
            relative_path: PathBuf::from("apps/android/app/src/main/res/values/styles.xml"),
            content: styles_xml(),
            kind: AndroidArtifactKind::ProjectFile,
            target: "android",
        },
        AndroidArtifact {
            relative_path: PathBuf::from(
                "apps/android/app/src/main/java/dev/dowe/generated/MainActivity.kt",
            ),
            content: main_activity(),
            kind: AndroidArtifactKind::Entrypoint,
            target: "android",
        },
        AndroidArtifact {
            relative_path: PathBuf::from(
                "apps/android/app/src/main/java/dev/dowe/generated/GeneratedViews.kt",
            ),
            content: generated_views_index(),
            kind: AndroidArtifactKind::GeneratedView,
            target: "android",
        },
        AndroidArtifact {
            relative_path: PathBuf::from(
                "apps/android/app/src/main/java/dev/dowe/generated/DoweRouting.kt",
            ),
            content: android_routing(routes),
            kind: AndroidArtifactKind::Routing,
            target: "android",
        },
        AndroidArtifact {
            relative_path: PathBuf::from(
                "apps/android/app/src/main/java/dev/dowe/generated/DoweLayouts.kt",
            ),
            content: android_layouts(),
            kind: AndroidArtifactKind::Layouts,
            target: "android",
        },
        AndroidArtifact {
            relative_path: PathBuf::from(
                "apps/android/app/src/main/java/dev/dowe/generated/DowePages.kt",
            ),
            content: generated_views(routes, font_config, &font_families, design_config),
            kind: AndroidArtifactKind::Pages,
            target: "android",
        },
        AndroidArtifact {
            relative_path: PathBuf::from(
                "apps/android/app/src/main/java/dev/dowe/generated/DoweEnvironment.kt",
            ),
            content: android_environment(environment),
            kind: AndroidArtifactKind::ProjectFile,
            target: "android",
        },
        AndroidArtifact {
            relative_path: PathBuf::from(
                "apps/android/app/src/main/java/dev/dowe/generated/DoweTheme.kt",
            ),
            content: android_theme(design_config),
            kind: AndroidArtifactKind::Theme,
            target: "android",
        },
        AndroidArtifact {
            relative_path: PathBuf::from(
                "apps/android/app/src/main/java/dev/dowe/generated/DoweResponsive.kt",
            ),
            content: android_responsive(),
            kind: AndroidArtifactKind::Responsive,
            target: "android",
        },
        AndroidArtifact {
            relative_path: PathBuf::from("apps/android/dev/AndroidManifest.xml"),
            content: dev_manifest(app_name, app_bundle),
            kind: AndroidArtifactKind::Manifest,
            target: "android",
        },
        AndroidArtifact {
            relative_path: PathBuf::from(
                "apps/android/dev/src/dev/dowe/generated/DoweDevActivity.java",
            ),
            content: dev_activity(routes, font_config, design_config, environment, app_bundle),
            kind: AndroidArtifactKind::DevEntrypoint,
            target: "android",
        },
    ];
    files.extend(android_translation_artifacts(translations));
    AndroidOutput { files }
}

fn android_translation_artifacts(catalog: &TranslationCatalog) -> Vec<AndroidArtifact> {
    catalog
        .locales
        .iter()
        .map(|locale| {
            let directory = if Some(locale.locale.as_str()) == catalog.default_locale.as_deref() {
                "values".to_string()
            } else {
                format!("values-{}", locale.locale)
            };
            AndroidArtifact {
                relative_path: PathBuf::from(format!(
                    "apps/android/app/src/main/res/{directory}/strings.xml"
                )),
                content: android_strings_xml(locale),
                kind: AndroidArtifactKind::Localization,
                target: "android",
            }
        })
        .collect()
}

fn android_strings_xml(locale: &dowe_components::TranslationLocale) -> String {
    let values = locale
        .values
        .iter()
        .map(|value| {
            format!(
                "    <string name=\"{}\">{}</string>",
                translation_resource_name(&value.key),
                escape_android_xml(&value.value)
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!("<resources>\n{values}\n</resources>\n")
}

fn escape_android_xml(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
        .replace('\'', "\\'")
}

fn android_environment(environment: &[(String, String)]) -> String {
    let mut values = environment
        .iter()
        .map(|(name, value)| format!("    const val {} = \"{}\"", name, escape_kotlin(value)))
        .collect::<Vec<_>>();
    if !environment.iter().any(|(name, _)| name == "BACKEND_URL") {
        values.push("    const val BACKEND_URL = \"\"".to_string());
    }
    let values = values.join("\n");
    format!(
        r#"package dev.dowe.generated

object DoweEnvironment {{
{values}
}}
"#
    )
}

fn generated_views_index() -> String {
    "package dev.dowe.generated\n".to_string()
}

fn android_routing(routes: &[ViewRoute]) -> String {
    let route_paths = routes
        .iter()
        .map(|route| format!("    \"{}\",", route.route_path))
        .collect::<Vec<_>>()
        .join("\n");
    let initial = routes_first_path(routes);
    let deep_links = routes
        .iter()
        .map(|route| {
            format!(
                "    \"dowe-dev://generated{}\",",
                if route.route_path == "/" {
                    "/"
                } else {
                    route.route_path.as_str()
                }
            )
        })
        .collect::<Vec<_>>()
        .join("\n");
    let sections = routes
        .iter()
        .map(|route| {
            let values = route
                .sections
                .iter()
                .map(|section| format!("\"{}\"", escape_kotlin(&section.id)))
                .collect::<Vec<_>>()
                .join(", ");
            format!("    \"{}\" to listOf({values}),", route.route_path)
        })
        .collect::<Vec<_>>()
        .join("\n");
    format!(
        r#"package dev.dowe.generated

object DoweRoutes {{
    const val initialPath = "{initial}"
    val paths = listOf(
{route_paths}
    )
    val sections: Map<String, List<String>> = mapOf(
{sections}
    )
    val deepLinks = listOf(
{deep_links}
    )
}}
"#
    )
}

fn routes_first_path(routes: &[ViewRoute]) -> &str {
    routes
        .first()
        .map(|route| route.route_path.as_str())
        .unwrap_or("/")
}

fn android_layouts() -> String {
    r#"package dev.dowe.generated

import androidx.compose.runtime.Composable

@Composable
fun DoweLayoutBoundary(content: @Composable () -> Unit) {
    content()
}
"#
    .to_string()
}

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

    defaultConfig {{
        applicationId = "{}"
        minSdk = 26
        targetSdk = 36
        versionCode = 1
        versionName = "0.1.0"
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
    implementation("androidx.compose.material3:material3:1.4.0")
}}
"#
,
        escape_kotlin(app_bundle)
    )
}

fn android_manifest(app_name: &str) -> String {
    format!(
        r#"<manifest xmlns:android="http://schemas.android.com/apk/res/android">
    <uses-permission android:name="android.permission.INTERNET" />
    <application android:theme="@style/AppTheme" android:label="{}" android:usesCleartextTraffic="true">
        <activity android:name=".MainActivity" android:exported="true" android:windowSoftInputMode="adjustResize">
            <intent-filter>
                <action android:name="android.intent.action.MAIN" />
                <category android:name="android.intent.category.LAUNCHER" />
            </intent-filter>
            <intent-filter>
                <action android:name="android.intent.action.VIEW" />
                <category android:name="android.intent.category.DEFAULT" />
                <category android:name="android.intent.category.BROWSABLE" />
                <data android:scheme="dowe-dev" android:host="generated" />
            </intent-filter>
        </activity>
    </application>
</manifest>
"#
,
        escape_android_xml(app_name)
    )
}

fn dev_manifest(app_name: &str, app_bundle: &str) -> String {
    format!(
        r#"<manifest xmlns:android="http://schemas.android.com/apk/res/android" package="{}">
    <uses-sdk android:minSdkVersion="26" android:targetSdkVersion="36" />
    <uses-permission android:name="android.permission.INTERNET" />
    <application android:theme="@android:style/Theme.Material.Light.NoActionBar" android:label="{}" android:usesCleartextTraffic="true">
        <activity android:name="dev.dowe.generated.DoweDevActivity" android:exported="true" android:windowSoftInputMode="adjustResize">
            <intent-filter>
                <action android:name="android.intent.action.MAIN" />
                <category android:name="android.intent.category.LAUNCHER" />
            </intent-filter>
            <intent-filter>
                <action android:name="android.intent.action.VIEW" />
                <category android:name="android.intent.category.DEFAULT" />
                <category android:name="android.intent.category.BROWSABLE" />
                <data android:scheme="dowe-dev" android:host="generated" />
            </intent-filter>
        </activity>
    </application>
</manifest>
"#
,
        app_bundle,
        escape_android_xml(app_name)
    )
}

fn styles_xml() -> String {
    r#"<resources>
    <style name="AppTheme" parent="android:style/Theme.Material.Light.NoActionBar" />
</resources>
"#
    .to_string()
}

fn main_activity() -> String {
    r#"package dev.dowe.generated

import android.content.Intent
import android.os.Bundle
import androidx.activity.enableEdgeToEdge
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue

class MainActivity : ComponentActivity() {
    private var incomingPath by mutableStateOf(DoweRoutes.initialPath)
    private var incomingFragment by mutableStateOf<String?>(null)
    private var incomingRequest by mutableStateOf(0)

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()
        applyIntentRoute(intent)
        setContent {
            DoweApp(incomingPath, incomingFragment, incomingRequest)
        }
    }

    override fun onNewIntent(intent: Intent) {
        super.onNewIntent(intent)
        setIntent(intent)
        applyIntentRoute(intent)
    }

    private fun applyIntentRoute(intent: Intent?) {
        val path = intent?.data?.path?.takeIf { DoweRoutes.paths.contains(it) } ?: DoweRoutes.initialPath
        incomingPath = path
        incomingFragment = intent?.data?.fragment?.takeIf { DoweRoutes.sections[path]?.contains(it) == true }
        incomingRequest += 1
    }
}
"#
    .to_string()
}
