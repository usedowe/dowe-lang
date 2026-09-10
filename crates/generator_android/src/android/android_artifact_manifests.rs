fn android_manifest(
    app_name: &str,
    has_app_icon: bool,
    uses_camera: bool,
    uses_microphone: bool,
) -> String {
    let icon_attributes = android_icon_attributes(has_app_icon);
    let capture_permissions = android_capture_permissions(uses_camera, uses_microphone);
    format!(
        r#"<manifest xmlns:android="http://schemas.android.com/apk/res/android">
    <uses-permission android:name="android.permission.INTERNET" />
    <uses-permission android:name="android.permission.POST_NOTIFICATIONS" />
{capture_permissions}
    <application android:theme="@style/AppTheme" android:label="{}" android:usesCleartextTraffic="true"{icon_attributes}>
        <meta-data android:name="dev.dowe.ir.schema" android:value="{}" />
        <activity android:name=".MainActivity" android:exported="true" android:windowSoftInputMode="adjustResize" android:supportsPictureInPicture="true" android:configChanges="screenSize|smallestScreenSize|screenLayout|orientation">
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
"#,
        escape_android_xml(app_name),
        dowe_components::VIEW_IR_SCHEMA_VERSION
    )
}

fn dev_manifest(
    app_name: &str,
    app_bundle: &str,
    has_app_icon: bool,
    uses_camera: bool,
    uses_microphone: bool,
) -> String {
    let icon_attributes = android_icon_attributes(has_app_icon);
    let capture_permissions = android_capture_permissions(uses_camera, uses_microphone);
    format!(
        r#"<manifest xmlns:android="http://schemas.android.com/apk/res/android" package="{}">
    <uses-sdk android:minSdkVersion="26" android:targetSdkVersion="36" />
    <uses-permission android:name="android.permission.INTERNET" />
    <uses-permission android:name="android.permission.POST_NOTIFICATIONS" />
{capture_permissions}
    <application android:theme="@android:style/Theme.Material.Light.NoActionBar" android:label="{}" android:usesCleartextTraffic="true"{icon_attributes}>
        <meta-data android:name="dev.dowe.ir.schema" android:value="{}" />
        <activity android:name="dev.dowe.generated.DoweDevHostActivity" android:exported="true" android:windowSoftInputMode="adjustResize" android:supportsPictureInPicture="true" android:configChanges="screenSize|smallestScreenSize|screenLayout|orientation">
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
"#,
        app_bundle,
        escape_android_xml(app_name),
        dowe_components::VIEW_IR_SCHEMA_VERSION
    )
}

fn android_capture_permissions(uses_camera: bool, uses_microphone: bool) -> String {
    [
        uses_camera.then_some("    <uses-permission android:name=\"android.permission.CAMERA\" />"),
        uses_microphone
            .then_some("    <uses-permission android:name=\"android.permission.RECORD_AUDIO\" />"),
    ]
    .into_iter()
    .flatten()
    .collect::<Vec<_>>()
    .join("\n")
}

fn android_notifications() -> String {
    r#"package dev.dowe.generated

import android.Manifest
import android.app.Activity
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.PendingIntent
import android.content.Context
import android.content.Intent
import android.content.pm.PackageManager
import android.net.Uri
import android.os.Build
import androidx.core.app.ActivityCompat

object DoweNotifications {
    private const val CHANNEL_ID = "dowe-default"
    private const val PERMISSION_REQUEST_CODE = 4101

    fun requestPermission(activity: Activity) {
        if (Build.VERSION.SDK_INT >= 33 &&
            ActivityCompat.checkSelfPermission(activity, Manifest.permission.POST_NOTIFICATIONS) != PackageManager.PERMISSION_GRANTED
        ) {
            ActivityCompat.requestPermissions(
                activity,
                arrayOf(Manifest.permission.POST_NOTIFICATIONS),
                PERMISSION_REQUEST_CODE
            )
        }
    }

    fun show(context: Context, id: String, title: String, body: String, route: String? = null) {
        require(id.isNotBlank() && title.isNotBlank() && body.isNotBlank())
        require(route == null || (route.startsWith("/") && !route.contains("//")))
        val manager = context.getSystemService(NotificationManager::class.java)
        if (Build.VERSION.SDK_INT >= 26) {
            manager.createNotificationChannel(
                NotificationChannel(CHANNEL_ID, "Dowe", NotificationManager.IMPORTANCE_DEFAULT)
            )
        }
        if (Build.VERSION.SDK_INT >= 33 &&
            ActivityCompat.checkSelfPermission(context, Manifest.permission.POST_NOTIFICATIONS) != PackageManager.PERMISSION_GRANTED
        ) return
        val intent = Intent(context, MainActivity::class.java).apply {
            route?.let { data = Uri.parse("dowe-dev://generated$it") }
            flags = Intent.FLAG_ACTIVITY_SINGLE_TOP or Intent.FLAG_ACTIVITY_CLEAR_TOP
        }
        val pendingIntent = PendingIntent.getActivity(
            context,
            id.hashCode(),
            intent,
            PendingIntent.FLAG_UPDATE_CURRENT or PendingIntent.FLAG_IMMUTABLE
        )
        val notification = android.app.Notification.Builder(context, CHANNEL_ID)
            .setSmallIcon(android.R.drawable.ic_dialog_info)
            .setContentTitle(title)
            .setContentText(body)
            .setAutoCancel(true)
            .setContentIntent(pendingIntent)
            .build()
        manager.notify(id, id.hashCode(), notification)
    }
}
"#
    .to_string()
}

