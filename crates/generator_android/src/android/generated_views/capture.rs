fn android_runtime_capture() -> &'static str {
    r#"@Composable
private fun DoweCamera(state: DoweReactiveState, facing: String, label: String, disabled: Boolean, onStart: String?, onCapture: String?, onError: String?, modifier: Modifier, shape: RoundedCornerShape, backgroundColor: Color, contentColor: Color) {
    val context = LocalContext.current
    val actionScope = rememberCoroutineScope()
    val launcher = rememberLauncherForActivityResult(ActivityResultContracts.StartActivityForResult()) { result ->
        val bitmap = result.data?.extras?.get("data") as? Bitmap
        if (bitmap == null) {
            onError?.let { actionScope.launch { state.run(it, mapOf("source" to "camera", "kind" to "error", "error" to "capture_failed")) } }
        } else {
            runCatching {
                val file = File.createTempFile("dowe-camera-", ".jpg", context.cacheDir)
                FileOutputStream(file).use { bitmap.compress(Bitmap.CompressFormat.JPEG, 92, it) }
                onCapture?.let { actionScope.launch { state.run(it, mapOf("source" to "camera", "kind" to "capture", "facing" to facing, "mimeType" to "image/jpeg", "url" to Uri.fromFile(file).toString(), "width" to bitmap.width, "height" to bitmap.height)) } }
            }.onFailure {
                onError?.let { actionScope.launch { state.run(it, mapOf("source" to "camera", "kind" to "error", "error" to "write_failed")) } }
            }
        }
    }
    val launchCamera: () -> Unit = {
        onStart?.let { actionScope.launch { state.run(it, mapOf("source" to "camera", "kind" to "start", "facing" to facing)) } }
        launcher.launch(Intent(MediaStore.ACTION_IMAGE_CAPTURE).apply {
            putExtra("android.intent.extras.CAMERA_FACING", if (facing == "user") 1 else 0)
        })
    }
    val permissionLauncher = rememberLauncherForActivityResult(ActivityResultContracts.RequestPermission()) { granted ->
        if (granted) {
            launchCamera()
        } else {
            onError?.let { actionScope.launch { state.run(it, mapOf("source" to "camera", "kind" to "error", "error" to "permission_denied")) } }
        }
    }
    Button(
        onClick = {
            if (ContextCompat.checkSelfPermission(context, Manifest.permission.CAMERA) == PackageManager.PERMISSION_GRANTED) {
                launchCamera()
            } else {
                permissionLauncher.launch(Manifest.permission.CAMERA)
            }
        },
        enabled = !disabled,
        modifier = modifier,
        shape = shape,
        colors = ButtonDefaults.buttonColors(containerColor = backgroundColor, contentColor = contentColor)
    ) { Text(label) }
}

@Composable
private fun DoweMicrophone(state: DoweReactiveState, label: String, maxDuration: Int?, disabled: Boolean, onStart: String?, onStop: String?, onError: String?, modifier: Modifier, shape: RoundedCornerShape, backgroundColor: Color, contentColor: Color) {
    val context = LocalContext.current
    val actionScope = rememberCoroutineScope()
    var recorder by remember { mutableStateOf<MediaRecorder?>(null) }
    var outputFile by remember { mutableStateOf<File?>(null) }
    var recording by remember { mutableStateOf(false) }
    var startedAt by remember { mutableStateOf(0L) }
    var elapsed by remember { mutableStateOf(0L) }
    val startRecording: () -> Unit = {
        runCatching {
            val file = File.createTempFile("dowe-microphone-", ".m4a", context.cacheDir)
            val next = MediaRecorder()
            next.setAudioSource(MediaRecorder.AudioSource.MIC)
            next.setOutputFormat(MediaRecorder.OutputFormat.MPEG_4)
            next.setAudioEncoder(MediaRecorder.AudioEncoder.AAC)
            next.setOutputFile(file.absolutePath)
            next.prepare()
            next.start()
            recorder = next
            outputFile = file
            startedAt = System.currentTimeMillis()
            elapsed = 0L
            recording = true
            onStart?.let { actionScope.launch { state.run(it, mapOf("source" to "microphone", "kind" to "start")) } }
        }.onFailure {
            onError?.let { actionScope.launch { state.run(it, mapOf("source" to "microphone", "kind" to "error", "error" to "unavailable")) } }
        }
    }
    val permissionLauncher = rememberLauncherForActivityResult(ActivityResultContracts.RequestPermission()) { granted ->
        if (granted) {
            startRecording()
        } else {
            onError?.let { actionScope.launch { state.run(it, mapOf("source" to "microphone", "kind" to "error", "error" to "permission_denied")) } }
        }
    }
    LaunchedEffect(recording, startedAt, maxDuration) {
        while (recording) {
            elapsed = (System.currentTimeMillis() - startedAt).coerceAtLeast(0L)
            if (maxDuration != null && elapsed >= maxDuration * 1000L) {
                recorder?.let { current -> runCatching { current.stop() } }
                recorder?.release()
                recorder = null
                recording = false
                outputFile?.let { file -> onStop?.let { actionScope.launch { state.run(it, mapOf("source" to "microphone", "kind" to "stop", "mimeType" to "audio/mp4", "url" to Uri.fromFile(file).toString(), "durationMs" to elapsed)) } } }
            }
            delay(250)
        }
    }
    DisposableEffect(Unit) {
        onDispose {
            recorder?.let { current -> runCatching { current.stop() }; current.release() }
            recorder = null
            recording = false
        }
    }
    Row(modifier = modifier, horizontalArrangement = Arrangement.spacedBy(12.dp), verticalAlignment = Alignment.CenterVertically) {
        Text(label, color = contentColor, fontWeight = FontWeight.SemiBold)
        Spacer(Modifier.weight(1f))
        Text("${elapsed / 60000}:${(elapsed / 1000 % 60).toString().padStart(2, '0')}", color = contentColor)
        Button(
            onClick = {
                if (recording) {
                    val duration = (System.currentTimeMillis() - startedAt).coerceAtLeast(0L)
                    recorder?.let { current -> runCatching { current.stop() } }
                    recorder?.release()
                    recorder = null
                    recording = false
                    outputFile?.let { file -> onStop?.let { actionScope.launch { state.run(it, mapOf("source" to "microphone", "kind" to "stop", "mimeType" to "audio/mp4", "url" to Uri.fromFile(file).toString(), "durationMs" to duration)) } } }
                } else if (ContextCompat.checkSelfPermission(context, Manifest.permission.RECORD_AUDIO) == PackageManager.PERMISSION_GRANTED) {
                    startRecording()
                } else {
                    permissionLauncher.launch(Manifest.permission.RECORD_AUDIO)
                }
            },
            enabled = !disabled,
            shape = shape,
            colors = ButtonDefaults.buttonColors(containerColor = backgroundColor, contentColor = contentColor)
        ) { Text(if (recording) "Stop" else label) }
    }
}
"#
}
