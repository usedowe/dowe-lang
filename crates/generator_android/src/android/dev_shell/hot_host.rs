fn dev_hot_host_activity() -> String {
    r#"package dev.dowe.generated;

import android.app.Activity;
import android.content.Intent;
import android.content.res.Configuration;
import android.os.Bundle;
import android.os.Handler;
import android.os.Looper;
import android.util.Log;
import android.widget.FrameLayout;
import android.widget.TextView;
import dalvik.system.DexClassLoader;
import java.io.ByteArrayOutputStream;
import java.io.File;
import java.io.FileOutputStream;
import java.io.InputStream;
import java.lang.reflect.Method;
import java.net.HttpURLConnection;
import java.net.URL;
import org.json.JSONObject;

@SuppressWarnings("deprecation")
public final class DoweDevHostActivity extends Activity {
    private static final String HMR_PREFERENCES = "dowe-hmr";
    private static final String HMR_ENDPOINT = "endpoint";
    private static final String HMR_VERSION = "version";
    private final Handler handler = new Handler(Looper.getMainLooper());
    private String endpoint = "";
    private String activeVersion = "";
    private String attemptedVersion = "";
    private Object activeModule;
    private Method activePath;
    private Method back;
    private Method intent;
    private Method pictureInPicture;
    private Method activityResult;
    private boolean running = true;

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        endpoint = resolveEndpoint(getIntent());
        FrameLayout loading = new FrameLayout(this);
        TextView label = new TextView(this);
        label.setText("Loading Dowe module");
        loading.addView(label);
        setContentView(loading);
        restoreCachedModule();
        poll();
    }

    @Override
    protected void onNewIntent(Intent nextIntent) {
        super.onNewIntent(nextIntent);
        setIntent(nextIntent);
        boolean resumePolling = endpoint == null || endpoint.isEmpty();
        endpoint = resolveEndpoint(nextIntent);
        if (resumePolling && endpoint != null && !endpoint.isEmpty()) {
            poll();
        }
        if (activeModule != null && intent != null) {
            try {
                intent.invoke(activeModule, nextIntent);
            } catch (Exception error) {
                Log.e("DoweHmr", "intent update failed", error);
            }
        }
    }

    @Override
    public void onPictureInPictureModeChanged(boolean active, Configuration configuration) {
        super.onPictureInPictureModeChanged(active, configuration);
        if (activeModule != null && pictureInPicture != null) {
            try {
                pictureInPicture.invoke(activeModule, active);
            } catch (Exception error) {
                Log.e("DoweHmr", "picture-in-picture update failed", error);
            }
        }
    }

    @Override
    protected void onActivityResult(int requestCode, int resultCode, Intent data) {
        super.onActivityResult(requestCode, resultCode, data);
        if (activeModule != null && activityResult != null) {
            try {
                activityResult.invoke(activeModule, requestCode, resultCode, data);
            } catch (Exception error) {
                Log.e("DoweHmr", "activity result dispatch failed", error);
            }
        }
    }

    @Override
    public void onBackPressed() {
        if (activeModule != null && back != null) {
            try {
                back.invoke(activeModule);
                return;
            } catch (Exception error) {
                Log.e("DoweHmr", "back dispatch failed", error);
            }
        }
        super.onBackPressed();
    }

    @Override
    protected void onDestroy() {
        running = false;
        handler.removeCallbacksAndMessages(null);
        super.onDestroy();
    }

    private String resolveEndpoint(Intent launchIntent) {
        String value = launchIntent == null ? null : launchIntent.getStringExtra("doweDevServer");
        if (value != null && !value.isEmpty()) {
            getSharedPreferences(HMR_PREFERENCES, MODE_PRIVATE)
                .edit()
                .putString(HMR_ENDPOINT, value).apply();
            return value;
        }
        String stored = getSharedPreferences(HMR_PREFERENCES, MODE_PRIVATE)
            .getString(HMR_ENDPOINT, "");
        return stored == null ? "" : stored;
    }

    private File moduleDirectory() throws Exception {
        File directory = new File(getFilesDir(), "dowe-modules");
        if (!directory.isDirectory() && !directory.mkdirs()) {
            throw new IllegalStateException("unable to create Android module directory");
        }
        return directory;
    }

    private File moduleFile(String version) throws Exception {
        return new File(moduleDirectory(), "dowe-module-" + version + ".dex");
    }

    private void restoreCachedModule() {
        try {
            String version = getSharedPreferences(HMR_PREFERENCES, MODE_PRIVATE)
                .getString(HMR_VERSION, "");
            if (version == null || version.isEmpty()) {
                return;
            }
            File module = moduleFile(version);
            if (!module.isFile()) {
                return;
            }
            if (!module.setReadOnly()) {
                throw new IllegalStateException("unable to protect cached Android module");
            }
            apply(module, version);
        } catch (Exception error) {
            Log.e("DoweHmr", "cached module restore failed", error);
        }
    }

    private String activeModulePath() {
        if (activeModule != null && activePath != null) {
            try {
                Object value = activePath.invoke(activeModule);
                if (value instanceof String && !((String) value).isEmpty()) {
                    return (String) value;
                }
            } catch (Exception error) {
                Log.e("DoweHmr", "route read failed", error);
            }
        }
        return null;
    }

    private void poll() {
        if (!running || endpoint == null || endpoint.isEmpty()) {
            return;
        }
        new Thread(() -> {
            try {
                JSONObject manifest = readJson(endpoint + "/_dowe/dev/modules/manifest.json");
                JSONObject android = manifest.getJSONObject("targets").optJSONObject("android");
                if (android != null) {
                    String version = android.getString("version");
                    String path = android.getString("path");
                    if (!version.equals(activeVersion) && !version.equals(attemptedVersion)) {
                        File module = download(endpoint + path, version);
                        handler.post(() -> apply(module, version));
                    }
                }
            } catch (Exception error) {
                Log.e("DoweHmr", "module poll failed", error);
            }
            handler.postDelayed(this::poll, 300);
        }).start();
    }

    private JSONObject readJson(String value) throws Exception {
        HttpURLConnection connection = (HttpURLConnection) new URL(value).openConnection();
        connection.setUseCaches(false);
        connection.setConnectTimeout(1000);
        connection.setReadTimeout(1000);
        try (InputStream input = connection.getInputStream()) {
            return new JSONObject(new String(readBytes(input), java.nio.charset.StandardCharsets.UTF_8));
        } finally {
            connection.disconnect();
        }
    }

    private File download(String value, String version) throws Exception {
        File target = moduleFile(version);
        if (target.isFile()) {
            if (!target.setReadOnly()) {
                throw new IllegalStateException("unable to protect Android module");
            }
            return target;
        }
        File staged = new File(moduleDirectory(), target.getName() + ".tmp");
        HttpURLConnection connection = (HttpURLConnection) new URL(value).openConnection();
        connection.setUseCaches(false);
        try (InputStream input = connection.getInputStream(); FileOutputStream output = new FileOutputStream(staged)) {
            byte[] buffer = new byte[16384];
            int length;
            while ((length = input.read(buffer)) >= 0) {
                output.write(buffer, 0, length);
            }
        } finally {
            connection.disconnect();
        }
        if (!staged.renameTo(target)) {
            throw new IllegalStateException("unable to publish Android module");
        }
        if (!target.setReadOnly()) {
            throw new IllegalStateException("unable to protect Android module");
        }
        return target;
    }

    private byte[] readBytes(InputStream input) throws Exception {
        ByteArrayOutputStream output = new ByteArrayOutputStream();
        byte[] buffer = new byte[8192];
        int length;
        while ((length = input.read(buffer)) >= 0) {
            output.write(buffer, 0, length);
        }
        return output.toByteArray();
    }

    private void apply(File file, String version) {
        attemptedVersion = version;
        try {
            boolean initialMount = activeModule == null;
            String path = initialMount ? null : activeModulePath();
            DexClassLoader loader = new DexClassLoader(file.getAbsolutePath(), getCodeCacheDir().getAbsolutePath(), null, getClassLoader());
            Class<?> type = loader.loadClass("dev.dowe.generated.DoweDevActivity");
            Object module = type.getConstructor(Activity.class).newInstance(this);
            Method mount = type.getMethod("mount", String.class, Intent.class);
            Method nextPath = type.getMethod("currentPath");
            Method nextBack = type.getMethod("handleBack");
            Method nextIntent = type.getMethod("handleIntent", Intent.class);
            Method nextPictureInPicture = type.getMethod("handlePictureInPictureMode", boolean.class);
            Method nextActivityResult = type.getMethod("handleActivityResult", int.class, int.class, Intent.class);
            mount.invoke(module, path, initialMount ? getIntent() : null);
            activeModule = module;
            activePath = nextPath;
            back = nextBack;
            intent = nextIntent;
            pictureInPicture = nextPictureInPicture;
            activityResult = nextActivityResult;
            activeVersion = version;
            getSharedPreferences(HMR_PREFERENCES, MODE_PRIVATE)
                .edit()
                .putString(HMR_VERSION, version).apply();
        } catch (Exception error) {
            Log.e("DoweHmr", "module apply failed", error);
        }
    }
}
"#
    .to_string()
}
