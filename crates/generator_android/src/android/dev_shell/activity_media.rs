fn append_dev_activity_media(output: &mut String) {
    output.push_str(
        r#"    private void doweOpenCamera(String facing, String onStart, String onCapture, String onError) {
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

"#,
    );
}
