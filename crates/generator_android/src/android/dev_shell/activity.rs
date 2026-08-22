struct DevActivitySources {
    core: String,
    shards: Vec<DevActivityShard>,
}

struct DevActivityShard {
    file_name: String,
    content: String,
}

fn dev_activity_sources(
    routes: &[ViewRoute],
    font_config: &FontConfig,
    font_families: &BTreeSet<FontFamily>,
    design_config: &DesignConfig,
    environment: &[(String, String)],
    app_bundle: &str,
) -> DevActivitySources {
    let has_phones = routes.iter().any(|route| {
        dev_tree_has_phone(&route.layout_tree) || dev_tree_has_phone(&route.page_tree)
    });
    let has_dynamic_icons = routes.iter().any(|route| {
        dowe_components::tree_has_dynamic_icon(&route.layout_tree)
            || dowe_components::tree_has_dynamic_icon(&route.page_tree)
    });
    let (layouts, route_layouts) = reusable_dev_layouts(routes);
    let route_classes = routes
        .iter()
        .map(|route| dev_route_class_name(&route.route_path))
        .collect::<Vec<_>>();
    let mut output = String::from(dev_activity_header());
    insert_dev_app_r_import(&mut output, app_bundle);
    output.push_str(&dev_design_constants(design_config));
    output.push_str("    private static final int DOWE_IMAGE_CROPPER_REQUEST = 5108;\n    private static final int DOWE_CAMERA_REQUEST = 5109;\n    private static final int DOWE_MICROPHONE_PERMISSION_REQUEST = 5110;\n    private static final int DOWE_CAMERA_PERMISSION_REQUEST = 5111;\n    private String doweImageCropperKey;\n    private String doweImageCropperAspect;\n    private int doweImageCropperMinWidth;\n    private int doweImageCropperMinHeight;\n    private int doweImageCropperMaxWidth;\n    private int doweImageCropperMaxHeight;\n    private String doweCameraOnCapture;\n    private String doweCameraOnError;\n    private String doweCameraFacing;\n    private String doweCameraPendingOnStart;\n    private String doweCameraPendingOnCapture;\n    private String doweCameraPendingOnError;\n    private String doweCameraPendingFacing;\n    private MediaRecorder doweMicrophoneRecorder;\n    private File doweMicrophoneFile;\n    private long doweMicrophoneStarted;\n    private String doweMicrophoneOnStop;\n    private String doweMicrophoneOnError;\n    private String doweMicrophonePendingOnStart;\n    private String doweMicrophonePendingOnStop;\n    private String doweMicrophonePendingOnError;\n    private int doweMicrophonePendingMaxDuration;\n");
    output.push_str("    private String doweImageCropperShapeName;\n");
    output.push_str("    private DoweVideoLayout dowePictureInPictureVideo;\n    private boolean dowePictureInPictureRestoreFullscreen;\n    private static final int DOWE_DROPZONE_REQUEST = 5107;\n    private String doweDropzoneKey;\n    private long doweDropzoneMaxSize = -1L;\n    private boolean doweDropzoneMultiple;\n    private boolean dowePinnedAppBarDockOnScroll;\n    private int dowePinnedAppBarColor;\n    private int dowePinnedAppBarHeight;\n    private float dowePinnedAppBarDockProgress;\n    private View dowePinnedAppBarPlaceholder;\n    private View dowePinnedAppBarDivider;\n    private ValueAnimator dowePinnedAppBarAnimator;\n");
    output.push_str(&format!(
        "    private final Activity doweActivity;\n    private Intent doweIntent;\n    private LinearLayout root;\n    private ScrollView scrollView;\n    private int viewportWidth;\n    private String currentPath = \"{}\";\n    private String currentFragment = null;\n    private String doweMountedPath = null;\n    private String doweMountedLayout = null;\n    private boolean externalOpen = false;\n    private Runnable doweDrawerNavigationClose = null;
    private PopupWindow doweActiveOverlay = null;
    private int doweOverlayRender = 0;
    private int doweOverlayClaimed = 0;
    private final ArrayList<DoweRouteEntry> backStack = new ArrayList<>();\n    private final HashMap<String, Object> doweState = new HashMap<>();\n    private final HashMap<String, Object> doweInitial = new HashMap<>();\n    private final HashMap<String, Boolean> doweSideNavMemory = new HashMap<>();\n    private final HashMap<String, String[]> doweSignalMetadata = new HashMap<>();\n    private final HashMap<String, Object> doweGlobalState = new HashMap<>();\n    private final HashMap<String, String> doweGlobalStorage = new HashMap<>();\n    private final HashMap<String, DoweAction> doweActions = new HashMap<>();\n    private final HashMap<String, DoweFormFieldMetadata[]> doweForms = new HashMap<>();\n    private final HashMap<String, View> sectionViews = new HashMap<>();\n    private final HashSet<String> doweLoaded = new HashSet<>();\n    private final HashSet<String> doweTouchedValidations = new HashSet<>();\n    private final HashSet<String> doweTouchedForms = new HashSet<>();\n\n",
        escape_java(routes_first_path(routes))
    ));
    output.push_str(
        r#"    private static final class DoweRouteEntry {
        private final String path;
        private final String fragment;

        private DoweRouteEntry(String path, String fragment) {
            this.path = path;
            this.fragment = fragment;
        }
    }

"#,
    );
    output.push_str("    private static final class DoweEnvironment {\n");
    for (name, value) in environment {
        output.push_str(&format!(
            "        private static final String {} = \"{}\";\n",
            name,
            escape_java(value)
        ));
    }
    if !environment.iter().any(|(name, _)| name == "BACKEND_URL") {
        output.push_str("        private static final String BACKEND_URL = \"\";\n");
    }
    output.push_str("    }\n\n");
    output.push_str(
        r#"    public DoweDevActivity(Activity activity) {
        super(activity, android.R.style.Theme_Material_Light_NoActionBar);
        doweActivity = activity;
    }

    public void mount(String preferredPath, Intent launchIntent) {
        doweIntent = launchIntent;
        String storedTheme = getSharedPreferences("dowe", 0).getString("theme-preference", DOWE_DEFAULT_THEME);
        doweApplyTheme(storedTheme == null ? DOWE_DEFAULT_THEME : storedTheme);
        doweConfigureWindow();
        FrameLayout background = new FrameLayout(this);
        background.setBackgroundColor(DOWE_BACKGROUND);
        background.setLayoutParams(new FrameLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.MATCH_PARENT));
        background.setClipChildren(false);
        background.setClipToPadding(false);
        root = new DoweLinearLayout(this);
        root.setOrientation(LinearLayout.VERTICAL);
        root.setGravity(Gravity.TOP | Gravity.START);
        root.setBackgroundColor(DOWE_BACKGROUND);
        root.setLayoutParams(new LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.MATCH_PARENT));
        scrollView = new ScrollView(this);
        scrollView.setFillViewport(true);
        scrollView.setClipChildren(false);
        scrollView.setClipToPadding(true);
        scrollView.setOnScrollChangeListener((view, scrollX, scrollY, oldScrollX, oldScrollY) -> doweUpdatePinnedAppBarDock(scrollY > doweDp(100), true));
        scrollView.addView(root, new ScrollView.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT));
        background.addView(scrollView, new FrameLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.MATCH_PARENT));
        viewportWidth = getResources().getConfiguration().screenWidthDp;
        doweInitializeState();
        if (doweCanRoute(preferredPath)) {
            currentPath = preferredPath;
        }
        doweApplyIntentRoute();
        doweActivity.setContentView(background);
        doweApplySystemInsets(scrollView);
        renderCurrentRoute();
    }

    private void doweSetTheme(String name) {
        getSharedPreferences("dowe", 0).edit().putString("theme-preference", name).apply();
        doweApplyTheme(name);
        doweApplySystemBarAppearance();
        root.setBackgroundColor(DOWE_BACKGROUND);
        ((View) scrollView.getParent()).setBackgroundColor(DOWE_BACKGROUND);
        renderCurrentRoute(false);
    }

    public String currentPath() {
        return currentPath;
    }

    public void handleBack() {
        doweBack();
    }

    public void handleIntent(Intent intent) {
        doweIntent = intent;
        doweApplyIntentRoute();
        renderCurrentRoute();
    }

    public void handleActivityResult(int requestCode, int resultCode, Intent data) {
        if (requestCode == DOWE_CAMERA_REQUEST) {
            if (resultCode == Activity.RESULT_OK && data != null && data.getExtras() != null && data.getExtras().get("data") instanceof Bitmap) {
                Bitmap bitmap = (Bitmap) data.getExtras().get("data");
                try {
                    File file = File.createTempFile("dowe-camera-", ".jpg", getCacheDir());
                    try (FileOutputStream stream = new FileOutputStream(file)) { bitmap.compress(Bitmap.CompressFormat.JPEG, 92, stream); }
                    Map<String, Object> item = new HashMap<>();
                    item.put("source", "camera"); item.put("kind", "capture"); item.put("facing", doweCameraFacing); item.put("mimeType", "image/jpeg"); item.put("url", Uri.fromFile(file).toString()); item.put("width", bitmap.getWidth()); item.put("height", bitmap.getHeight());
                    if (doweCameraOnCapture != null) doweRunAction(doweCameraOnCapture, item);
                } catch (Exception error) {
                    if (doweCameraOnError != null) { Map<String, Object> item = new HashMap<>(); item.put("source", "camera"); item.put("kind", "error"); item.put("error", "write_failed"); doweRunAction(doweCameraOnError, item); }
                }
            } else if (doweCameraOnError != null) {
                Map<String, Object> item = new HashMap<>(); item.put("source", "camera"); item.put("kind", "error"); item.put("error", "cancelled"); doweRunAction(doweCameraOnError, item);
            }
            return;
        }
        if (requestCode == DOWE_IMAGE_CROPPER_REQUEST && resultCode == Activity.RESULT_OK && data != null && data.getData() != null && doweImageCropperKey != null) {
            Uri uri = data.getData();
            new Thread(() -> {
                try (InputStream input = getContentResolver().openInputStream(uri)) {
                    Bitmap bitmap = BitmapFactory.decodeStream(input);
                    if (bitmap != null) runOnUiThread(() -> doweShowImageCropperEditor(bitmap, doweImageCropperKey, doweImageCropperAspect, doweImageCropperShapeName, doweImageCropperMinWidth, doweImageCropperMinHeight, doweImageCropperMaxWidth, doweImageCropperMaxHeight));
                } catch (Exception ignored) {}
            }).start();
            return;
        }
        if (requestCode != DOWE_DROPZONE_REQUEST || resultCode != Activity.RESULT_OK || data == null || doweDropzoneKey == null) {
            return;
        }
        ArrayList<Uri> uris = new ArrayList<>();
        if (data.getClipData() != null) {
            for (int index = 0; index < data.getClipData().getItemCount(); index++) {
                uris.add(data.getClipData().getItemAt(index).getUri());
            }
        } else if (data.getData() != null) {
            uris.add(data.getData());
        }
        ArrayList<String> files = new ArrayList<>();
        if (doweDropzoneMultiple) {
            Object previous = doweState.get(doweDropzoneKey);
            if (previous instanceof ArrayList<?>) {
                for (Object file : (ArrayList<?>) previous) {
                    files.add(String.valueOf(file));
                }
            }
        }
        for (Uri uri : uris) {
            long size = doweDropzoneFileSize(uri);
            if (doweDropzoneMaxSize >= 0L && size >= 0L && size > doweDropzoneMaxSize) {
                continue;
            }
            String label = doweDropzoneFileLabel(uri, size);
            if (!files.contains(label)) {
                files.add(label);
            }
            if (!doweDropzoneMultiple) {
                break;
            }
        }
        doweState.put(doweDropzoneKey, files);
        runOnUiThread(() -> renderCurrentRoute(false));
    }

    public void handlePermissionResult(int requestCode, String[] permissions, int[] grantResults) {
        if (requestCode == DOWE_CAMERA_PERMISSION_REQUEST) {
            String onStart = doweCameraPendingOnStart;
            String onCapture = doweCameraPendingOnCapture;
            String onError = doweCameraPendingOnError;
            String facing = doweCameraPendingFacing;
            doweCameraPendingOnStart = null;
            doweCameraPendingOnCapture = null;
            doweCameraPendingOnError = null;
            doweCameraPendingFacing = null;
            if (grantResults != null && grantResults.length > 0 && grantResults[0] == PackageManager.PERMISSION_GRANTED) {
                doweOpenCameraIntent(facing, onStart, onCapture, onError);
            } else if (onError != null) {
                Map<String, Object> item = new HashMap<>(); item.put("source", "camera"); item.put("kind", "error"); item.put("error", "permission_denied"); doweRunAction(onError, item);
            }
            return;
        }
        if (requestCode != DOWE_MICROPHONE_PERMISSION_REQUEST) return;
        String onStart = doweMicrophonePendingOnStart;
        String onStop = doweMicrophonePendingOnStop;
        String onError = doweMicrophonePendingOnError;
        int maxDuration = doweMicrophonePendingMaxDuration;
        doweMicrophonePendingOnStart = null;
        doweMicrophonePendingOnStop = null;
        doweMicrophonePendingOnError = null;
        doweMicrophonePendingMaxDuration = 0;
        if (grantResults != null && grantResults.length > 0 && grantResults[0] == PackageManager.PERMISSION_GRANTED) {
            doweStartMicrophoneRecording(onStart, onStop, onError, maxDuration);
        } else if (onError != null) {
            Map<String, Object> item = new HashMap<>(); item.put("source", "microphone"); item.put("kind", "error"); item.put("error", "permission_denied"); doweRunAction(onError, item);
        }
    }

    private void doweOpenCamera(String facing, String onStart, String onCapture, String onError) {
        doweCameraFacing = facing;
        doweCameraOnCapture = onCapture;
        doweCameraOnError = onError;
        if (Build.VERSION.SDK_INT >= 23 && doweActivity.checkSelfPermission(Manifest.permission.CAMERA) != PackageManager.PERMISSION_GRANTED) {
            doweCameraPendingOnStart = onStart;
            doweCameraPendingOnCapture = onCapture;
            doweCameraPendingOnError = onError;
            doweCameraPendingFacing = facing;
            doweActivity.requestPermissions(new String[]{Manifest.permission.CAMERA}, DOWE_CAMERA_PERMISSION_REQUEST);
            return;
        }
        doweOpenCameraIntent(facing, onStart, onCapture, onError);
    }

    private void doweOpenCameraIntent(String facing, String onStart, String onCapture, String onError) {
        doweCameraFacing = facing;
        doweCameraOnCapture = onCapture;
        doweCameraOnError = onError;
        Intent camera = new Intent("android.media.action.IMAGE_CAPTURE");
        camera.putExtra("android.intent.extras.CAMERA_FACING", "user".equals(facing) ? 1 : 0);
        if (camera.resolveActivity(getPackageManager()) == null) {
            if (onError != null) { Map<String, Object> item = new HashMap<>(); item.put("source", "camera"); item.put("kind", "error"); item.put("error", "unavailable"); doweRunAction(onError, item); }
            return;
        }
        if (onStart != null) { Map<String, Object> item = new HashMap<>(); item.put("source", "camera"); item.put("kind", "start"); item.put("facing", facing); doweRunAction(onStart, item, () -> {}); }
        doweActivity.startActivityForResult(camera, DOWE_CAMERA_REQUEST);
    }

    private void doweStartMicrophone(String onStart, String onStop, String onError, int maxDuration) {
        if (doweMicrophoneRecorder != null) return;
        if (Build.VERSION.SDK_INT >= 23 && doweActivity.checkSelfPermission(Manifest.permission.RECORD_AUDIO) != PackageManager.PERMISSION_GRANTED) {
            doweMicrophonePendingOnStart = onStart;
            doweMicrophonePendingOnStop = onStop;
            doweMicrophonePendingOnError = onError;
            doweMicrophonePendingMaxDuration = maxDuration;
            doweActivity.requestPermissions(new String[]{Manifest.permission.RECORD_AUDIO}, DOWE_MICROPHONE_PERMISSION_REQUEST);
            return;
        }
        doweStartMicrophoneRecording(onStart, onStop, onError, maxDuration);
    }

    private void doweStartMicrophoneRecording(String onStart, String onStop, String onError, int maxDuration) {
        if (doweMicrophoneRecorder != null) return;
        try {
            doweMicrophoneFile = File.createTempFile("dowe-microphone-", ".m4a", getCacheDir());
            doweMicrophoneRecorder = new MediaRecorder();
            doweMicrophoneRecorder.setAudioSource(MediaRecorder.AudioSource.MIC);
            doweMicrophoneRecorder.setOutputFormat(MediaRecorder.OutputFormat.MPEG_4);
            doweMicrophoneRecorder.setAudioEncoder(MediaRecorder.AudioEncoder.AAC);
            doweMicrophoneRecorder.setOutputFile(doweMicrophoneFile.getAbsolutePath());
            doweMicrophoneRecorder.prepare();
            doweMicrophoneRecorder.start();
            doweMicrophoneStarted = System.currentTimeMillis();
            doweMicrophoneOnStop = onStop;
            doweMicrophoneOnError = onError;
            if (onStart != null) { Map<String, Object> item = new HashMap<>(); item.put("source", "microphone"); item.put("kind", "start"); doweRunAction(onStart, item, () -> {}); }
            if (maxDuration > 0) new Handler(Looper.getMainLooper()).postDelayed(() -> doweStopMicrophone(), maxDuration * 1000L);
        } catch (Exception error) {
            doweMicrophoneRecorder = null;
            if (onError != null) { Map<String, Object> item = new HashMap<>(); item.put("source", "microphone"); item.put("kind", "error"); item.put("error", "unavailable"); doweRunAction(onError, item); }
        }
    }

    private void doweStopMicrophone() {
        if (doweMicrophoneRecorder == null) return;
        long duration = Math.max(0L, System.currentTimeMillis() - doweMicrophoneStarted);
        try { doweMicrophoneRecorder.stop(); } catch (Exception ignored) {}
        doweMicrophoneRecorder.release();
        doweMicrophoneRecorder = null;
        if (doweMicrophoneOnStop != null && doweMicrophoneFile != null) { Map<String, Object> item = new HashMap<>(); item.put("source", "microphone"); item.put("kind", "stop"); item.put("mimeType", "audio/mp4"); item.put("url", Uri.fromFile(doweMicrophoneFile).toString()); item.put("durationMs", duration); doweRunAction(doweMicrophoneOnStop, item); }
    }

    private void doweOpenDropzonePicker(String key, String accept, boolean multiple, long maxSize) {
        doweDropzoneKey = key;
        doweDropzoneMaxSize = maxSize;
        doweDropzoneMultiple = multiple;
        Intent picker = new Intent(Intent.ACTION_OPEN_DOCUMENT);
        picker.addCategory(Intent.CATEGORY_OPENABLE);
        String[] mimeTypes = doweDropzoneMimeTypes(accept);
        picker.setType(mimeTypes.length == 1 ? mimeTypes[0] : "*/*");
        picker.putExtra(Intent.EXTRA_MIME_TYPES, mimeTypes);
        picker.putExtra(Intent.EXTRA_ALLOW_MULTIPLE, multiple);
        doweActivity.startActivityForResult(picker, DOWE_DROPZONE_REQUEST);
    }

    private void doweOpenImageCropperPicker(String key, String accept, String aspect, String shape, int minWidth, int minHeight, int maxWidth, int maxHeight) {
        doweImageCropperKey = key;
        doweImageCropperAspect = aspect;
        doweImageCropperShapeName = shape;
        doweImageCropperMinWidth = minWidth;
        doweImageCropperMinHeight = minHeight;
        doweImageCropperMaxWidth = maxWidth;
        doweImageCropperMaxHeight = maxHeight;
        Intent picker = new Intent(Intent.ACTION_OPEN_DOCUMENT);
        picker.addCategory(Intent.CATEGORY_OPENABLE);
        String[] mimeTypes = doweDropzoneMimeTypes(accept);
        picker.setType(mimeTypes.length == 1 ? mimeTypes[0] : "*/*");
        picker.putExtra(Intent.EXTRA_MIME_TYPES, mimeTypes);
        doweActivity.startActivityForResult(picker, DOWE_IMAGE_CROPPER_REQUEST);
    }

    private String[] doweDropzoneMimeTypes(String accept) {
        if (accept == null || accept.trim().isEmpty()) {
            return new String[]{"*/*"};
        }
        String[] values = accept.split(",");
        ArrayList<String> types = new ArrayList<>();
        for (String value : values) {
            String trimmed = value.trim();
            if (!trimmed.isEmpty()) {
                types.add(trimmed);
            }
        }
        return types.isEmpty() ? new String[]{"*/*"} : types.toArray(new String[0]);
    }

    private long doweDropzoneFileSize(Uri uri) {
        android.database.Cursor cursor = getContentResolver().query(uri, new String[]{OpenableColumns.SIZE}, null, null, null);
        if (cursor == null) {
            return -1L;
        }
        try {
            if (cursor.moveToFirst() && !cursor.isNull(0)) {
                return cursor.getLong(0);
            }
        } finally {
            cursor.close();
        }
        return -1L;
    }

    private String doweDropzoneFileLabel(Uri uri, long size) {
        String name = uri.getLastPathSegment();
        android.database.Cursor cursor = getContentResolver().query(uri, new String[]{OpenableColumns.DISPLAY_NAME}, null, null, null);
        if (cursor != null) {
            try {
                if (cursor.moveToFirst() && !cursor.isNull(0)) {
                    name = cursor.getString(0);
                }
            } finally {
                cursor.close();
            }
        }
        String label = name == null || name.isEmpty() ? "Selected file" : name;
        return size < 0L ? label : label + " (" + doweDropzoneSizeLabel(size) + ")";
    }

    private String doweDropzoneSizeLabel(long size) {
        if (size >= 1024L * 1024L * 1024L) return (size / (1024L * 1024L * 1024L)) + " GB";
        if (size >= 1024L * 1024L) return (size / (1024L * 1024L)) + " MB";
        if (size >= 1024L) return (size / 1024L) + " KB";
        return size + " Bytes";
    }

    private String doweDropzoneText(String key, String placeholder) {
        Object value = doweState.get(key);
        if (!(value instanceof ArrayList<?>) || ((ArrayList<?>) value).isEmpty()) {
            return "Upload\\n" + placeholder;
        }
        StringBuilder text = new StringBuilder("Selected files");
        for (Object file : (ArrayList<?>) value) {
            text.append("\\n").append(String.valueOf(file));
        }
        return text.toString();
    }

    private Window getWindow() {
        return doweActivity.getWindow();
    }

    private Intent getIntent() {
        return doweIntent;
    }

    private void runOnUiThread(Runnable action) {
        doweActivity.runOnUiThread(action);
    }

    private void doweConfigureWindow() {
        getWindow().setStatusBarColor(Color.TRANSPARENT);
        getWindow().setNavigationBarColor(Color.TRANSPARENT);
        if (Build.VERSION.SDK_INT >= 29) {
            getWindow().setNavigationBarContrastEnforced(false);
        }
        if (Build.VERSION.SDK_INT >= 30) {
            getWindow().setDecorFitsSystemWindows(false);
        }
        doweApplySystemBarAppearance();
    }

    private void doweApplySystemBarAppearance() {
        boolean useDarkIcons = Color.luminance(DOWE_BACKGROUND) > 0.179f;
        if (Build.VERSION.SDK_INT >= 30) {
            int mask = android.view.WindowInsetsController.APPEARANCE_LIGHT_STATUS_BARS |
                android.view.WindowInsetsController.APPEARANCE_LIGHT_NAVIGATION_BARS;
            getWindow().getInsetsController().setSystemBarsAppearance(useDarkIcons ? mask : 0, mask);
        } else {
            int visibility = View.SYSTEM_UI_FLAG_LAYOUT_STABLE |
                View.SYSTEM_UI_FLAG_LAYOUT_FULLSCREEN |
                View.SYSTEM_UI_FLAG_LAYOUT_HIDE_NAVIGATION;
            if (useDarkIcons) {
                visibility |= View.SYSTEM_UI_FLAG_LIGHT_STATUS_BAR |
                    View.SYSTEM_UI_FLAG_LIGHT_NAVIGATION_BAR;
            }
            getWindow().getDecorView().setSystemUiVisibility(visibility);
        }
    }

    private void doweApplySystemInsets(View view) {
        view.setOnApplyWindowInsetsListener((target, insets) -> {
            int previousLeft = target.getPaddingLeft();
            int previousTop = target.getPaddingTop();
            int previousRight = target.getPaddingRight();
            int previousBottom = target.getPaddingBottom();
            if (Build.VERSION.SDK_INT >= 30) {
                Insets safe = insets.getInsets(WindowInsets.Type.systemBars() | WindowInsets.Type.displayCutout());
                target.setPadding(safe.left, safe.top, safe.right, safe.bottom);
            } else {
                target.setPadding(
                    insets.getSystemWindowInsetLeft(),
                    insets.getSystemWindowInsetTop(),
                    insets.getSystemWindowInsetRight(),
                    insets.getSystemWindowInsetBottom()
                );
            }
            if (view == scrollView && (previousLeft != target.getPaddingLeft()
                    || previousTop != target.getPaddingTop()
                    || previousRight != target.getPaddingRight()
                    || previousBottom != target.getPaddingBottom())) {
                target.post(() -> renderCurrentRoute(false));
            }
            doweRelayoutPinnedAppBar();
            return insets;
        });
        view.requestApplyInsets();
    }

    private void renderCurrentRoute() {
        renderCurrentRoute(true);
    }

    private void renderCurrentRoute(boolean scrollToFragment) {
        doweOverlayRender++;
        root.removeAllViews();
        View pinnedAppBar = ((ViewGroup) scrollView.getParent()).findViewWithTag("dowe-pinned-appbar");
        if (pinnedAppBar != null) {
            ((ViewGroup) scrollView.getParent()).removeView(pinnedAppBar);
        }
        View pinnedAppBarSafeArea = ((ViewGroup) scrollView.getParent()).findViewWithTag("dowe-pinned-appbar-safe-area");
        if (pinnedAppBarSafeArea != null) {
            ((ViewGroup) scrollView.getParent()).removeView(pinnedAppBarSafeArea);
        }
        View pinnedAppBarBottomSafeArea = ((ViewGroup) scrollView.getParent()).findViewWithTag("dowe-pinned-appbar-bottom-safe-area");
        if (pinnedAppBarBottomSafeArea != null) {
            ((ViewGroup) scrollView.getParent()).removeView(pinnedAppBarBottomSafeArea);
        }
        View pinnedAppBarDivider = ((ViewGroup) scrollView.getParent()).findViewWithTag("dowe-pinned-appbar-divider");
        if (pinnedAppBarDivider != null) {
            ((ViewGroup) scrollView.getParent()).removeView(pinnedAppBarDivider);
        }
        if (dowePinnedAppBarAnimator != null) {
            dowePinnedAppBarAnimator.cancel();
            dowePinnedAppBarAnimator = null;
        }
        dowePinnedAppBarDockOnScroll = false;
        dowePinnedAppBarPlaceholder = null;
        dowePinnedAppBarDivider = null;
        View fixedFab = ((ViewGroup) scrollView.getParent()).findViewWithTag("dowe-fixed-fab");
        if (fixedFab != null) {
            ((ViewGroup) scrollView.getParent()).removeView(fixedFab);
        }
        sectionViews.clear();
        externalOpen = false;
"#,
    );

    for (index, (route, class_name)) in routes.iter().zip(&route_classes).enumerate() {
        let branch = if index == 0 { "if" } else { "else if" };
        output.push_str(&format!(
            "        {branch} (\"{}\".equals(currentPath)) {{\n            {class_name}.render(this, root);\n        }}\n",
            escape_java(&route.route_path)
        ));
    }

    if let Some((route, class_name)) = routes.first().zip(route_classes.first()) {
        output.push_str(&format!(
            "        else {{\n            currentPath = \"{}\";\n            {class_name}.render(this, root);\n        }}\n",
            escape_java(&route.route_path)
        ));
    }

    output.push_str(
        "        if (doweActiveOverlay != null && doweActiveOverlay.isShowing() && doweOverlayClaimed != doweOverlayRender) {\n            doweActiveOverlay.dismiss();\n        }\n        doweAutoload();\n        if (scrollToFragment) {\n            if (currentFragment == null) {\n                scrollView.scrollTo(0, 0);\n            } else {\n                doweScrollToFragment();\n            }\n        }\n    }\n\n",
    );

    output.push_str("    private void doweInitializeState() {\n");
    for class_name in &route_classes {
        output.push_str(&format!("        {class_name}.initialize(this);\n"));
    }
    output.push_str("    }\n\n    private void doweAutoload() {\n");
    for class_name in &route_classes {
        output.push_str(&format!("        {class_name}.autoload(this);\n"));
    }
    output.push_str("    }\n\n");

    output.push_str(&dev_activity_navigation(routes_first_path(routes)));

    if routes.is_empty() {
        output.push_str("        return false;\n");
    } else {
        let route_checks = routes
            .iter()
            .map(|route| format!("\"{}\".equals(path)", escape_java(&route.route_path)))
            .collect::<Vec<_>>()
            .join(" || ");
        output.push_str(&format!("        return {route_checks};\n"));
    }

    output.push_str(
        "    }\n\n    private boolean doweCanSection(String path, String fragment) {\n        if (fragment == null) {\n            return true;\n        }\n",
    );
    for route in routes {
        let section_checks = route
            .sections
            .iter()
            .map(|section| format!("\"{}\".equals(fragment)", escape_java(&section.id)))
            .collect::<Vec<_>>()
            .join(" || ");
        output.push_str(&format!(
            "        if (\"{}\".equals(path)) {{\n            return {};\n        }}\n",
            escape_java(&route.route_path),
            if section_checks.is_empty() {
                "false".to_string()
            } else {
                section_checks
            }
        ));
    }
    output.push_str("        return false;\n    }\n\n");

    output.push_str(dev_activity_layout_widgets());
    output.push_str(dev_activity_flex_layout());
    output.push_str(dev_activity_grid_layout());
    if has_dynamic_icons {
        output.push_str(dev_activity_dynamic_icon_runtime());
    }
    output.push_str(dev_activity_svg_parser());
    output.push_str(dev_activity_svg_view());
    output.push_str(dev_activity_drawables_media());
    output.push_str(dev_activity_image_cropper());
    output.push_str(dev_activity_candlestick_runtime());
    output.push_str(dev_activity_chart_runtime());
    output.push_str(dev_activity_canvas_runtime());
    output.push_str(dev_activity_code_and_forms());
    if has_phones {
        output.push_str(dev_activity_phone_flag_runtime());
    } else {
        output.push_str(dev_activity_empty_phone_flag_runtime());
    }
    output.push_str(dev_activity_responsive_helpers());
    output = output.replace(
        "__DOWE_ANDROID_DEV_FONT_SUPPORT__",
        &android_dev_font_support(font_families),
    );
    output = output.replace(
        "__DOWE_DEFAULT_FONT__",
        font_config
            .default_family
            .catalog_entry()
            .android_family_name,
    );
    output = output.replace("__DOWE_JAVA_REACTIVE_RUNTIME__", dev_java_reactive_runtime());
    output = output.replace(
        "__DOWE_SIDE_NAV_SUBMENU_ARROW_PATH__",
        SIDE_NAV_SUBMENU_ARROW_PATH,
    );

    let mut shards = routes
        .iter()
        .zip(&route_layouts)
        .zip(&route_classes)
        .map(|((route, layout_index), class_name)| DevActivityShard {
            file_name: format!("{class_name}.java"),
            content: dev_route_shard(route, *layout_index, class_name, app_bundle),
        })
        .collect::<Vec<_>>();
    shards.extend(
        layouts
            .iter()
            .enumerate()
            .map(|(index, layout)| DevActivityShard {
                file_name: format!("{}.java", dev_layout_class_name(index)),
                content: dev_layout_shard(layout, index, app_bundle),
            }),
    );
    if has_phones {
        shards.extend(dev_phone_flag_shards(app_bundle));
    }
    if has_dynamic_icons {
        shards.extend(dev_dynamic_icon_shards(app_bundle));
    }

    DevActivitySources {
        core: expose_dev_activity_members(output),
        shards,
    }
}
