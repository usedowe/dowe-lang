fn android_runtime_media_accordion_carousel() -> &'static str {
    r##"@Composable
private fun DoweAccordion(multiple: Boolean, variant: String, defaultOpenIds: Set<String>, modifier: Modifier, backgroundColor: Color, contentColor: Color, borderColor: Color?, itemBackgroundColor: Color, itemBorderColor: Color, itemBorderAlpha: Float, radius: Dp, content: @Composable (Set<String>, (String) -> Unit) -> Unit) {
    var openIds by remember(multiple, defaultOpenIds) { mutableStateOf(defaultOpenIds) }
    val toggleItem: (String) -> Unit = { id ->
        openIds = if (id in openIds) {
            openIds - id
        } else if (multiple) {
            openIds + id
        } else {
            setOf(id)
        }
    }
    Column(
        modifier = modifier
            .clip(RoundedCornerShape(radius))
            .background(backgroundColor)
            .then(if (borderColor == null) Modifier else Modifier.border(1.dp, borderColor, RoundedCornerShape(radius)))
            .padding(if (variant == "ghost" || variant == "line") 0.dp else 4.dp),
        verticalArrangement = Arrangement.spacedBy(if (variant == "ghost" || variant == "line") 0.dp else 8.dp)
    ) {
        CompositionLocalProvider(LocalContentColor provides contentColor) {
            content(openIds, toggleItem)
        }
    }
}

@Composable
private fun DoweAccordionItem(label: String, disabled: Boolean, open: Boolean, backgroundColor: Color, borderColor: Color, borderAlpha: Float, radius: Dp, onToggle: () -> Unit, arrowIcon: @Composable () -> Unit, content: @Composable () -> Unit) {
    val itemShape = RoundedCornerShape(radius)
    Column(modifier = Modifier.fillMaxWidth().clip(itemShape).background(backgroundColor).then(if (borderAlpha == 0f) Modifier else if (radius == 0.dp) Modifier.drawBehind { drawLine(borderColor.copy(alpha = borderAlpha), Offset(0f, size.height - 0.5.dp.toPx()), Offset(size.width, size.height - 0.5.dp.toPx()), strokeWidth = 1.dp.toPx()) } else Modifier.border(1.dp, borderColor.copy(alpha = borderAlpha), itemShape)).alpha(if (disabled) 0.5f else 1f)) {
        Row(
            modifier = Modifier.fillMaxWidth().clickable(enabled = !disabled, onClick = onToggle).padding(horizontal = 16.dp, vertical = 12.dp),
            verticalAlignment = Alignment.CenterVertically,
            horizontalArrangement = Arrangement.SpaceBetween
        ) {
            Text(label, fontSize = 15.sp, lineHeight = 20.sp, fontWeight = FontWeight.Bold, modifier = Modifier.weight(1f))
            Box(modifier = Modifier.size(20.dp).graphicsLayer { rotationZ = if (open) 90f else 0f }, contentAlignment = Alignment.Center) {
                arrowIcon()
            }
        }
        AnimatedVisibility(visible = open, enter = fadeIn(tween(160)) + expandVertically(tween(160)), exit = fadeOut(tween(160)) + shrinkVertically(tween(160))) {
            Column(modifier = Modifier.fillMaxWidth().padding(horizontal = 16.dp, vertical = 12.dp), verticalArrangement = Arrangement.spacedBy(8.dp)) {
                content()
            }
        }
    }
}

private data class DoweCarouselSlideSpec(val id: String, val content: @Composable () -> Unit)

@Composable
private fun DoweCarousel(variant: String, slides: List<DoweCarouselSlideSpec>, autoplay: Boolean, autoplayInterval: Int, disableLoop: Boolean, hideControls: Boolean, hideIndicators: Boolean, showNavigation: Boolean, showCounter: Boolean, orientation: String, size: String, indicatorType: String, title: String?, slideWidth: Int?, slideHeight: Int?, slidesPerView: Int, gap: Int, modifier: Modifier, accentColor: Color) {
    val listState = rememberLazyListState()
    val scope = rememberCoroutineScope()
    val slideCount = slides.size
    val currentIndex by remember {
        derivedStateOf {
            val layout = listState.layoutInfo
            val center = (layout.viewportStartOffset + layout.viewportEndOffset) / 2
            layout.visibleItemsInfo.minByOrNull { item -> kotlin.math.abs(item.offset + item.size / 2 - center) }?.index ?: 0
        }
    }
    val layoutInfo = listState.layoutInfo
    val viewportCenter = (layoutInfo.viewportStartOffset + layoutInfo.viewportEndOffset) / 2
    val viewportSize = max(1, layoutInfo.viewportEndOffset - layoutInfo.viewportStartOffset)
    fun slidePhase(index: Int): Float {
        val item = layoutInfo.visibleItemsInfo.firstOrNull { it.index == index } ?: return 1f
        return ((item.offset + item.size / 2 - viewportCenter).toFloat() / viewportSize.toFloat()).coerceIn(-1f, 1f)
    }
    val moveTo: (Int) -> Unit = { requested ->
        val target = when {
            requested < 0 && !disableLoop -> slideCount - 1
            requested >= slideCount && !disableLoop -> 0
            else -> min(slideCount - 1, max(0, requested))
        }
        if (slideCount > 0) scope.launch { listState.animateScrollToItem(target) }
    }
    LaunchedEffect(autoplay, autoplayInterval, currentIndex, slideCount) {
        if (autoplay && slideCount > 1) {
            delay(max(500, autoplayInterval).toLong())
            moveTo(currentIndex + 1)
        }
    }
    Column(modifier = modifier, verticalArrangement = Arrangement.spacedBy(12.dp)) {
        if (title != null) Text(title, fontWeight = FontWeight.Bold, color = accentColor)
        BoxWithConstraints(modifier = Modifier.fillMaxWidth().clipToBounds()) {
            val viewportWidth = maxWidth
            val resolvedWidth = when {
                slideWidth != null -> slideWidth.dp
                slidesPerView > 1 -> (viewportWidth - gap.dp * (slidesPerView - 1)) / slidesPerView
                variant == "simple" || variant == "masonry" || variant == "rtl" || variant == "sticky" -> minOf(280.dp, viewportWidth * 0.84f)
                else -> viewportWidth
            }
            val shouldSnap = variant !in listOf("simple", "masonry", "rtl", "sticky")
            val snapBehavior = rememberSnapFlingBehavior(lazyListState = listState)
            val freeBehavior = ScrollableDefaults.flingBehavior()
            Box(modifier = Modifier.fillMaxWidth()) {
                if (orientation == "vertical") {
                    LazyColumn(
                        modifier = Modifier.fillMaxWidth().heightIn(max = 560.dp),
                        state = listState,
                        verticalArrangement = Arrangement.spacedBy(gap.dp),
                        flingBehavior = if (shouldSnap) snapBehavior else freeBehavior
                    ) {
                        itemsIndexed(slides, key = { _, slide -> slide.id }) { index, slide ->
                            DoweCarouselSlide(variant = variant, index = index, phase = slidePhase(index), orientation = orientation, slideWidth = viewportWidth, slideHeight = slideHeight) { slide.content() }
                        }
                    }
                } else {
                    LazyRow(
                        modifier = Modifier.fillMaxWidth(),
                        state = listState,
                        reverseLayout = variant == "rtl",
                        horizontalArrangement = Arrangement.spacedBy(gap.dp),
                        flingBehavior = if (shouldSnap) snapBehavior else freeBehavior
                    ) {
                        itemsIndexed(slides, key = { _, slide -> slide.id }) { index, slide ->
                            DoweCarouselSlide(variant = variant, index = index, phase = slidePhase(index), orientation = orientation, slideWidth = resolvedWidth, slideHeight = slideHeight) { slide.content() }
                        }
                    }
                }
                if (showNavigation) {
                    Row(modifier = Modifier.fillMaxWidth().align(Alignment.Center), horizontalArrangement = Arrangement.SpaceBetween) {
                        TextButton(modifier = Modifier.size(36.dp), enabled = !disableLoop || currentIndex > 0, colors = ButtonDefaults.textButtonColors(contentColor = accentColor), contentPadding = PaddingValues(0.dp), onClick = { moveTo(currentIndex - 1) }) { Text("‹", fontSize = 22.sp) }
                        TextButton(modifier = Modifier.size(36.dp), enabled = !disableLoop || currentIndex < slideCount - 1, colors = ButtonDefaults.textButtonColors(contentColor = accentColor), contentPadding = PaddingValues(0.dp), onClick = { moveTo(currentIndex + 1) }) { Text("›", fontSize = 22.sp) }
                    }
                }
            }
        }
        if (!hideControls || variant == "controls") {
            Row(modifier = Modifier.fillMaxWidth(), horizontalArrangement = Arrangement.SpaceBetween) {
                TextButton(modifier = Modifier.height(32.dp), enabled = !disableLoop || currentIndex > 0, colors = ButtonDefaults.textButtonColors(contentColor = accentColor), contentPadding = PaddingValues(horizontal = 8.dp, vertical = 0.dp), onClick = { moveTo(currentIndex - 1) }) { Text("Previous", fontSize = 14.sp) }
                TextButton(modifier = Modifier.height(32.dp), enabled = !disableLoop || currentIndex < slideCount - 1, colors = ButtonDefaults.textButtonColors(contentColor = accentColor), contentPadding = PaddingValues(horizontal = 8.dp, vertical = 0.dp), onClick = { moveTo(currentIndex + 1) }) { Text("Next", fontSize = 14.sp) }
            }
        }
        if (!hideIndicators || variant == "dots" || variant == "thumbnails") {
            Row(modifier = Modifier.fillMaxWidth().horizontalScroll(rememberScrollState()), horizontalArrangement = Arrangement.spacedBy(8.dp)) {
                repeat(slideCount) { index ->
                    TextButton(modifier = Modifier.heightIn(min = 28.dp), colors = ButtonDefaults.textButtonColors(contentColor = if (index == currentIndex) accentColor else accentColor.copy(alpha = 0.45f)), contentPadding = PaddingValues(horizontal = 4.dp, vertical = 0.dp), onClick = { moveTo(index) }) { Text(if (variant == "thumbnails") "Slide ${index + 1}" else if (indicatorType == "dot" || variant == "dots") "•" else "${index + 1}", fontSize = if (variant == "thumbnails") 12.sp else 16.sp) }
                }
            }
        }
        if (showCounter) Text("${currentIndex + 1} / $slideCount", color = accentColor)
    }
}

@Composable
private fun DoweCarouselSlide(variant: String, index: Int, phase: Float, orientation: String, slideWidth: Dp, slideHeight: Int?, content: @Composable () -> Unit) {
    val distance = kotlin.math.abs(phase).coerceIn(0f, 1f)
    val effect = when (variant) {
        "coverFlow" -> Modifier.graphicsLayer { rotationY = phase * 24f; scaleX = 1f - distance * 0.1f; scaleY = scaleX; alpha = 1f - distance * 0.22f; cameraDistance = 24f * density }
        "stories" -> Modifier.graphicsLayer { rotationY = phase * 30f; scaleX = 1f - distance * 0.1f; scaleY = scaleX; alpha = 1f - distance * 0.22f; cameraDistance = 24f * density }
        "smartStack" -> Modifier.graphicsLayer { rotationZ = phase * 1.5f; scaleX = 1f - distance * 0.055f; scaleY = scaleX; translationY = distance * 8f }
        "cardStack" -> Modifier.graphicsLayer { scaleX = 1f - distance * 0.055f; scaleY = scaleX; translationY = distance * 8f }
        "flipbook" -> Modifier.graphicsLayer { rotationY = phase * 52f; scaleX = 1f - distance * 0.1f; scaleY = scaleX; alpha = 1f - distance * 0.22f; cameraDistance = 24f * density }
        "slideshow" -> Modifier.graphicsLayer { translationX = if (orientation == "vertical") 0f else phase * 24f; translationY = if (orientation == "vertical") phase * 24f else 0f; alpha = 1f - distance * 0.12f }
        else -> Modifier
    }
    val size = if (orientation == "vertical") Modifier.fillMaxWidth() else Modifier.width(slideWidth)
    Box(modifier = size.then(if (slideHeight == null) Modifier else Modifier.height(slideHeight.dp)).then(effect)) { content() }
}

"##
}
