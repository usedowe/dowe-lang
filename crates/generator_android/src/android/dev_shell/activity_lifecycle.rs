fn append_dev_activity_lifecycle(output: &mut String) {
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
        doweUpdateSafeAreaColors();
        doweApplySystemBarAppearance();
        doweActivity.setContentView(background);
        doweApplySystemInsets(scrollView);
        renderCurrentRoute();
    }

    private void doweSetTheme(String name) {
        getSharedPreferences("dowe", 0).edit().putString("theme-preference", name).apply();
        doweApplyTheme(name);
        doweUpdateSafeAreaColors();
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
        String previousPath = currentPath;
        doweIntent = intent;
        doweApplyIntentRoute();
        doweUpdateSafeAreaColors();
        doweApplySystemBarAppearance();
        if (!previousPath.equals(currentPath)) {
            doweStartPageTransition();
        }
        renderCurrentRoute();
        if (!previousPath.equals(currentPath)) {
            doweFinishPageTransition();
        }
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

"#,
    );
}
