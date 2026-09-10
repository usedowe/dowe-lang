fn collect_media_node_classes(node: &ViewNode, classes: &mut BTreeSet<String>) {
    match node {
            ViewNode::Audio { props } => {
                classes.extend(variant_classes("media", &props.style));
                classes.extend([
                    "media-button".to_string(),
                    "media-icon".to_string(),
                    "media-content".to_string(),
                    "media-waveform".to_string(),
                    "media-bars".to_string(),
                    "media-bar".to_string(),
                    "media-footer".to_string(),
                    "media-time".to_string(),
                    "media-subtitle".to_string(),
                    "media-avatar".to_string(),
                ]);
            }
            ViewNode::Camera { props } => {
                classes.extend(variant_classes("camera", &props.style));
                classes.extend([
                    "camera-preview".to_string(),
                    "camera-placeholder".to_string(),
                    "camera-controls".to_string(),
                    "camera-button".to_string(),
                    "camera-status".to_string(),
                ]);
            }
            ViewNode::Microphone { props } => {
                classes.extend(variant_classes("microphone", &props.style));
                classes.extend([
                    "microphone-panel".to_string(),
                    "microphone-label".to_string(),
                    "microphone-status".to_string(),
                    "microphone-time".to_string(),
                    "microphone-controls".to_string(),
                    "microphone-button".to_string(),
                ]);
            }
            ViewNode::Image { props } => {
                classes.extend(variant_classes("image", &props.style));
                classes.extend([
                    props.aspect.as_str().to_string(),
                    format!("fit-{}", props.object_fit.as_str()),
                    "image-element".to_string(),
                    "image-controls".to_string(),
                    "image-actions".to_string(),
                    "image-action".to_string(),
                ]);
            }
            ViewNode::Accordion { props, items } => {
                classes.extend(variant_classes("accordion", &props.style));
                classes.extend([
                    "accordion-item".to_string(),
                    "accordion-header".to_string(),
                    "accordion-start".to_string(),
                    "accordion-label".to_string(),
                    "accordion-end".to_string(),
                    "accordion-arrow".to_string(),
                    "accordion-content".to_string(),
                    "accordion-content-inner".to_string(),
                ]);
                for item in items {
                    for child in &item.children {
                        collect_classes(child, classes);
                    }
                }
            }
            ViewNode::Carousel { props, slides } => {
                classes.extend(variant_classes("carousel", &props.style));
                classes.extend([
                    "carousel-header".to_string(),
                    "carousel-title".to_string(),
                    "carousel-viewport".to_string(),
                    "carousel-container".to_string(),
                    "carousel-slide".to_string(),
                    "carousel-controls".to_string(),
                    "carousel-control".to_string(),
                    "carousel-indicators".to_string(),
                    "carousel-indicator".to_string(),
                    "carousel-counter".to_string(),
                    "carousel-nav".to_string(),
                    "carousel-thumbnails".to_string(),
                    "carousel-thumbnail".to_string(),
                    "is-simple".to_string(),
                    "is-snapping".to_string(),
                    "is-masonry".to_string(),
                    "is-rtl".to_string(),
                    "is-sticky".to_string(),
                    "is-controls".to_string(),
                    "is-dots".to_string(),
                    "is-thumbnails".to_string(),
                    "is-cover-flow".to_string(),
                    "is-slideshow".to_string(),
                    "is-stories".to_string(),
                    "is-smart-stack".to_string(),
                    "is-card-stack".to_string(),
                    "is-flipbook".to_string(),
                ]);
                for slide in slides {
                    for child in &slide.children {
                        collect_classes(child, classes);
                    }
                }
            }
        _ => {}
    }
}
