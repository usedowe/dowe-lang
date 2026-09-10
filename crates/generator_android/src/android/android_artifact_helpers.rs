fn android_tree_has_camera(node: &ViewNode) -> bool {
    if matches!(node, ViewNode::Camera { .. }) {
        return true;
    }
    node_child_groups(node)
        .into_iter()
        .flatten()
        .any(android_tree_has_camera)
}

fn android_tree_has_microphone(node: &ViewNode) -> bool {
    if matches!(node, ViewNode::Microphone { .. }) {
        return true;
    }
    node_child_groups(node)
        .into_iter()
        .flatten()
        .any(android_tree_has_microphone)
}

fn android_icon_attributes(has_app_icon: bool) -> &'static str {
    if has_app_icon {
        " android:icon=\"@mipmap/ic_launcher\" android:roundIcon=\"@mipmap/ic_launcher_round\""
    } else {
        ""
    }
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
import android.content.res.Configuration
import android.os.Bundle
import androidx.activity.enableEdgeToEdge
import androidx.activity.ComponentActivity
import androidx.activity.compose.setContent
import androidx.compose.runtime.SideEffect
import androidx.compose.runtime.getValue
import androidx.compose.runtime.mutableStateOf
import androidx.compose.runtime.setValue
import androidx.compose.ui.graphics.luminance
import androidx.core.view.WindowCompat

class MainActivity : ComponentActivity() {
    private var incomingPath by mutableStateOf(DoweRoutes.initialPath)
    private var incomingFragment by mutableStateOf<String?>(null)
    private var incomingRequest by mutableStateOf(0)

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        enableEdgeToEdge()
        restoreThemePreference()
        applyIntentRoute(intent)
        setContent {
            val useDarkStatusBarIcons = doweSafeAreaTopColor(incomingPath).luminance() > 0.179f
            val useDarkNavigationBarIcons = doweSafeAreaBottomColor(incomingPath).luminance() > 0.179f
            SideEffect {
                WindowCompat.getInsetsController(window, window.decorView).apply {
                    isAppearanceLightStatusBars = useDarkStatusBarIcons
                    isAppearanceLightNavigationBars = useDarkNavigationBarIcons
                }
            }
            DoweApp(incomingPath, incomingFragment, incomingRequest)
        }
    }

    override fun onNewIntent(intent: Intent) {
        super.onNewIntent(intent)
        setIntent(intent)
        applyIntentRoute(intent)
    }

    override fun onPictureInPictureModeChanged(isInPictureInPictureMode: Boolean, newConfig: Configuration) {
        super.onPictureInPictureModeChanged(isInPictureInPictureMode, newConfig)
        doweHandleVideoPictureInPictureMode(isInPictureInPictureMode)
    }

    private fun restoreThemePreference() {
        val storedTheme = getSharedPreferences("dowe", MODE_PRIVATE)
            .getString("theme-preference", DoweThemeModule.defaultTheme)
            ?: DoweThemeModule.defaultTheme
        DoweDesign.applyTheme(storedTheme)
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
