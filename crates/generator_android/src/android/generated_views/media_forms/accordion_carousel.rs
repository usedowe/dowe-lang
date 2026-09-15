fn android_runtime_media_accordion_carousel() -> &'static str {
    r##"@Composable
private fun DoweAccordion(multiple: Boolean, variant: String, defaultOpenIds: Set<String>, modifier: Modifier, backgroundColor: Color, contentColor: Color, titleColor: Color, borderColor: Color?, itemBackgroundColor: Color, itemBorderColor: Color, itemBorderAlpha: Float, radius: Dp, content: @Composable (Set<String>, (String) -> Unit) -> Unit) {
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
        CompositionLocalProvider(LocalContentColor provides contentColor, LocalDoweTitleColor provides titleColor) {
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

private data class DoweCarouselGeometry(
    val contentGap: Int,
    val viewportPadding: Int,
    val verticalViewportHeight: Int,
    val slideFractionPercent: Int,
    val slideMaxWidth: Int?,
)

private data class DoweCarouselSlideSpec(val id: String, val content: @Composable () -> Unit)

private data class DoweCarouselControlContract(
    val navigationSize: Int,
    val navigationInset: Int,
    val controlSize: Int,
    val controlGap: Int,
    val indicatorGap: Int,
    val indicatorHeight: Int,
    val indicatorInactiveWidth: Int,
    val indicatorActiveWidth: Int,
    val indicatorDotSize: Int,
    val indicatorDotActiveScalePercent: Int,
)

@Composable
private fun DoweCarousel(variant: String, snap: Boolean, control: DoweCarouselControlContract, geometry: DoweCarouselGeometry, slides: List<DoweCarouselSlideSpec>, autoplay: Boolean, autoplayInterval: Int, disableLoop: Boolean, hideControls: Boolean, hideIndicators: Boolean, showNavigation: Boolean, showCounter: Boolean, orientation: String, size: String, indicatorType: String, title: String?, slideWidth: Int?, slideHeight: Int?, slidesPerView: Int, gap: Int, previousIcon: @Composable () -> Unit, nextIcon: @Composable () -> Unit, modifier: Modifier, accentColor: Color) {
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
    val autoplayIndex = androidx.compose.runtime.rememberUpdatedState(currentIndex)
    LaunchedEffect(autoplay, autoplayInterval, disableLoop, slideCount) {
        if (autoplay && slideCount > 1) {
            while (true) {
                delay(max(500, autoplayInterval).toLong())
                if (!listState.isScrollInProgress && !(disableLoop && autoplayIndex.value >= slideCount - 1)) moveTo(autoplayIndex.value + 1)
            }
        }
    }
    Column(modifier = modifier.semantics { contentDescription = "${title ?: "Carousel"}, slide ${currentIndex + 1} of $slideCount" }, verticalArrangement = Arrangement.spacedBy(geometry.contentGap.dp)) {
        if (title != null) Text(title, color = accentColor, fontSize = 24.sp, lineHeight = 29.sp, fontWeight = FontWeight.Bold)
        BoxWithConstraints(modifier = Modifier.fillMaxWidth().padding(geometry.viewportPadding.dp).clipToBounds()) {
            val viewportWidth = maxWidth
            val resolvedWidth = when {
                slideWidth != null -> slideWidth.dp
                geometry.slideMaxWidth != null -> minOf(geometry.slideMaxWidth.dp, viewportWidth * (geometry.slideFractionPercent / 100f))
                slidesPerView > 1 -> maxOf(0.dp, (viewportWidth - gap.dp * (slidesPerView - 1)) / slidesPerView)
                else -> viewportWidth
            }
            val shouldSnap = snap
            val snapBehavior = rememberSnapFlingBehavior(lazyListState = listState)
            val freeBehavior = ScrollableDefaults.flingBehavior()
            Box(modifier = Modifier.fillMaxWidth()) {
                if (orientation == "vertical") {
                    LazyColumn(
                        modifier = Modifier.fillMaxWidth().height(geometry.verticalViewportHeight.dp),
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
                    if (orientation == "vertical") {
                        Column(modifier = Modifier.fillMaxHeight().align(Alignment.Center).padding(vertical = control.navigationInset.dp), verticalArrangement = Arrangement.SpaceBetween, horizontalAlignment = Alignment.CenterHorizontally) {
                            DoweIconButton(enabled = !disableLoop || currentIndex > 0, dimension = control.navigationSize.dp, backgroundColor = DoweDesign.surface, contentColor = accentColor, borderColor = accentColor.copy(alpha = 0.24f), label = "Previous slide", onClick = { moveTo(currentIndex - 1) }) { previousIcon() }
                            DoweIconButton(enabled = !disableLoop || currentIndex < slideCount - 1, dimension = control.navigationSize.dp, backgroundColor = DoweDesign.surface, contentColor = accentColor, borderColor = accentColor.copy(alpha = 0.24f), label = "Next slide", onClick = { moveTo(currentIndex + 1) }) { nextIcon() }
                        }
                    } else {
                        Row(modifier = Modifier.fillMaxWidth().align(Alignment.Center).padding(horizontal = control.navigationInset.dp), horizontalArrangement = Arrangement.SpaceBetween) {
                            DoweIconButton(enabled = !disableLoop || currentIndex > 0, dimension = control.navigationSize.dp, backgroundColor = DoweDesign.surface, contentColor = accentColor, borderColor = accentColor.copy(alpha = 0.24f), label = "Previous slide", onClick = { moveTo(currentIndex - 1) }) { previousIcon() }
                            DoweIconButton(enabled = !disableLoop || currentIndex < slideCount - 1, dimension = control.navigationSize.dp, backgroundColor = DoweDesign.surface, contentColor = accentColor, borderColor = accentColor.copy(alpha = 0.24f), label = "Next slide", onClick = { moveTo(currentIndex + 1) }) { nextIcon() }
                        }
                    }
                }
            }
        }
        if (!hideControls || variant == "controls" || !hideIndicators || variant == "dots" || variant == "thumbnails" || showCounter) {
            Row(modifier = Modifier.fillMaxWidth(), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(control.controlGap.dp, Alignment.CenterHorizontally)) {
                if (!hideControls || variant == "controls") {
                    DoweIconButton(enabled = !disableLoop || currentIndex > 0, dimension = control.controlSize.dp, backgroundColor = DoweDesign.surface, contentColor = accentColor, borderColor = accentColor.copy(alpha = 0.24f), label = "Previous slide", onClick = { moveTo(currentIndex - 1) }) { previousIcon() }
                }
                if (!hideIndicators || variant == "dots" || variant == "thumbnails") {
                    Row(modifier = Modifier.weight(1f, fill = false).horizontalScroll(rememberScrollState()), verticalAlignment = Alignment.CenterVertically, horizontalArrangement = Arrangement.spacedBy(control.indicatorGap.dp)) {
                        slides.forEachIndexed { index, slide ->
                            if (variant == "thumbnails") {
                                TextButton(modifier = Modifier.heightIn(min = 28.dp), colors = ButtonDefaults.textButtonColors(contentColor = if (index == currentIndex) accentColor else accentColor.copy(alpha = 0.45f)), contentPadding = PaddingValues(horizontal = 4.dp, vertical = 0.dp), onClick = { moveTo(index) }) { Text(slide.id, fontSize = 12.sp) }
                            } else {
                                DowePaginationIndicator(active = index == currentIndex, dot = indicatorType == "dot" || variant == "dots", enabled = true, inactiveWidth = control.indicatorInactiveWidth.dp, activeWidth = control.indicatorActiveWidth.dp, height = control.indicatorHeight.dp, dotSize = control.indicatorDotSize.dp, dotActiveScalePercent = control.indicatorDotActiveScalePercent, color = accentColor, label = "Go to slide ${index + 1}") { moveTo(index) }
                            }
                        }
                    }
                }
                if (showCounter) Text("${currentIndex + 1} / $slideCount", color = accentColor, maxLines = 1)
                if (!hideControls || variant == "controls") {
                    DoweIconButton(enabled = !disableLoop || currentIndex < slideCount - 1, dimension = control.controlSize.dp, backgroundColor = DoweDesign.surface, contentColor = accentColor, borderColor = accentColor.copy(alpha = 0.24f), label = "Next slide", onClick = { moveTo(currentIndex + 1) }) { nextIcon() }
                }
            }
        }
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
