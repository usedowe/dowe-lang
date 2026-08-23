fn android_runtime_media_upload() -> &'static str {
    r##"private data class DowePickedFile(val uri: Uri, val name: String, val size: Long?)

private fun doweDropzoneMimeTypes(accept: String?): Array<String> {
    val values = accept
        ?.split(",")
        ?.map { it.trim() }
        ?.filter { it.isNotEmpty() }
        ?.toTypedArray()
    return if (values.isNullOrEmpty()) arrayOf("*/*") else values
}

private fun dowePickedFile(context: android.content.Context, uri: Uri, maxSize: Long?): DowePickedFile? {
    var name = uri.lastPathSegment ?: "Selected file"
    var size: Long? = null
    context.contentResolver.query(uri, arrayOf(OpenableColumns.DISPLAY_NAME, OpenableColumns.SIZE), null, null, null)?.use { cursor ->
        if (cursor.moveToFirst()) {
            val nameIndex = cursor.getColumnIndex(OpenableColumns.DISPLAY_NAME)
            val sizeIndex = cursor.getColumnIndex(OpenableColumns.SIZE)
            if (nameIndex >= 0) name = cursor.getString(nameIndex) ?: name
            if (sizeIndex >= 0 && !cursor.isNull(sizeIndex)) size = cursor.getLong(sizeIndex)
        }
    }
    if (maxSize != null && size != null && size > maxSize) return null
    return DowePickedFile(uri, name, size)
}

private fun dowePickedFileSize(size: Long?): String {
    if (size == null || size < 0) return ""
    val units = arrayOf("Bytes", "KB", "MB", "GB")
    var value = size.toDouble()
    var index = 0
    while (value >= 1024 && index < units.lastIndex) {
        value /= 1024
        index += 1
    }
    return "%.1f %s".format(java.util.Locale.US, value, units[index])
}

@Composable
private fun DoweDropzone(label: String?, placeholder: String, accept: String?, multiple: Boolean, maxSize: Long?, disabled: Boolean, helpText: String?, errorText: String?, size: String, modifier: Modifier, backgroundColor: Color, contentColor: Color, borderColor: Color?) {
    val context = LocalContext.current
    val selectedFiles = remember { mutableStateListOf<DowePickedFile>() }
    val multiplePicker = rememberLauncherForActivityResult(ActivityResultContracts.OpenMultipleDocuments()) { uris ->
        val known = selectedFiles.map { it.uri }.toHashSet()
        uris.mapNotNull { uri -> dowePickedFile(context, uri, maxSize) }
            .filter { it.uri !in known }
            .forEach { selectedFiles.add(it) }
    }
    val singlePicker = rememberLauncherForActivityResult(ActivityResultContracts.OpenDocument()) { uri ->
        uri?.let { pickedUri ->
            selectedFiles.clear()
            dowePickedFile(context, pickedUri, maxSize)?.let { selectedFiles.add(it) }
        }
    }
    val height = when (size) {
        "sm" -> 128.dp
        "lg" -> 256.dp
        else -> 192.dp
    }
    val launchPicker = {
        if (!disabled) {
            if (multiple) multiplePicker.launch(doweDropzoneMimeTypes(accept))
            else singlePicker.launch(doweDropzoneMimeTypes(accept))
        }
    }
    Column(modifier = modifier, verticalArrangement = Arrangement.spacedBy(8.dp)) {
        if (label != null) {
            Text(label, fontSize = 14.sp, fontWeight = FontWeight.SemiBold, color = contentColor)
        }
        Box(
            modifier = Modifier
                .fillMaxWidth()
                .height(height)
                .clip(RoundedCornerShape(12.dp))
                .background(backgroundColor)
                .border(2.dp, borderColor ?: contentColor.copy(alpha = 0.55f), RoundedCornerShape(12.dp))
                .clickable(enabled = !disabled, onClick = launchPicker)
                .then(if (disabled) Modifier.graphicsLayer { alpha = 0.5f } else Modifier),
            contentAlignment = Alignment.Center
        ) {
            Column(horizontalAlignment = Alignment.CenterHorizontally, verticalArrangement = Arrangement.spacedBy(8.dp)) {
                Text(if (selectedFiles.isEmpty()) "Upload" else "Selected files", color = contentColor.copy(alpha = 0.55f), fontWeight = FontWeight.SemiBold)
                if (selectedFiles.isEmpty()) {
                    Text(placeholder, color = contentColor.copy(alpha = 0.7f), fontSize = 14.sp)
                } else {
                    Column(horizontalAlignment = Alignment.CenterHorizontally) {
                        selectedFiles.take(3).forEach { file ->
                            Text(file.name, color = contentColor, fontSize = 14.sp, maxLines = 1)
                            val formattedSize = dowePickedFileSize(file.size)
                            if (formattedSize.isNotEmpty()) Text(formattedSize, color = contentColor.copy(alpha = 0.7f), fontSize = 12.sp)
                        }
                        if (selectedFiles.size > 3) Text("+${selectedFiles.size - 3} more", color = contentColor.copy(alpha = 0.7f), fontSize = 12.sp)
                    }
                }
            }
        }
        if (errorText != null || helpText != null) {
            Text(errorText ?: helpText.orEmpty(), fontSize = 12.sp, color = if (errorText != null) DoweDesign.danger else contentColor.copy(alpha = 0.7f))
        }
    }
}
private fun doweControlHeight(size: String): Dp {
    return when (size) {
        "sm" -> 32.dp
        "lg" -> 48.dp
        else -> 40.dp
    }
}

private fun doweControlSwatchSize(size: String): Dp {
    return when (size) {
        "sm" -> 20.dp
        "lg" -> 32.dp
        else -> 24.dp
    }
}

private fun doweHexColor(value: String, fallback: Color): Color {
    return try {
        Color(android.graphics.Color.parseColor(value))
    } catch (error: IllegalArgumentException) {
        fallback
    }
}

"##
}
