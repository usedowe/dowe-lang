fn dev_activity_initial_output(
    routes: &[ViewRoute],
    design_config: &DesignConfig,
    environment: &[(String, String)],
    app_bundle: &str,
) -> String {
    let mut output = String::from(dev_activity_header());
    insert_dev_app_r_import(&mut output, app_bundle);
    output.push_str(&dev_design_constants(design_config));
    output.push_str("    private static final int DOWE_IMAGE_CROPPER_REQUEST = 5108;\n    private static final int DOWE_CAMERA_REQUEST = 5109;\n    private static final int DOWE_MICROPHONE_PERMISSION_REQUEST = 5110;\n    private static final int DOWE_CAMERA_PERMISSION_REQUEST = 5111;\n    private String doweImageCropperKey;\n    private String doweImageCropperAspect;\n    private int doweImageCropperMinWidth;\n    private int doweImageCropperMinHeight;\n    private int doweImageCropperMaxWidth;\n    private int doweImageCropperMaxHeight;\n    private String doweCameraOnCapture;\n    private String doweCameraOnError;\n    private String doweCameraFacing;\n    private String doweCameraPendingOnStart;\n    private String doweCameraPendingOnCapture;\n    private String doweCameraPendingOnError;\n    private String doweCameraPendingFacing;\n    private MediaRecorder doweMicrophoneRecorder;\n    private File doweMicrophoneFile;\n    private long doweMicrophoneStarted;\n    private String doweMicrophoneOnStop;\n    private String doweMicrophoneOnError;\n    private String doweMicrophonePendingOnStart;\n    private String doweMicrophonePendingOnStop;\n    private String doweMicrophonePendingOnError;\n    private int doweMicrophonePendingMaxDuration;\n");
    output.push_str("    private String doweImageCropperShapeName;\n");
    output.push_str("    private DoweVideoLayout dowePictureInPictureVideo;\n    private boolean dowePictureInPictureRestoreFullscreen;\n    private static final int DOWE_DROPZONE_REQUEST = 5107;\n    private String doweDropzoneKey;\n    private long doweDropzoneMaxSize = -1L;\n    private boolean doweDropzoneMultiple;\n    private boolean dowePinnedAppBarDockOnScroll;\n    private int dowePinnedAppBarColor;\n    private int dowePinnedAppBarHeight;\n    private float dowePinnedAppBarDockProgress;\n    private View dowePinnedAppBarPlaceholder;\n    private View dowePinnedAppBarDivider;\n    private ValueAnimator dowePinnedAppBarAnimator;\n");
    output.push_str("    private int doweSafeAreaTopColor = DOWE_BACKGROUND;\n    private int doweSafeAreaBottomColor = DOWE_BACKGROUND;\n");
    output.push_str(&format!(
        "    private final Activity doweActivity;\n    private Intent doweIntent;\n    private LinearLayout root;\n    private ScrollView scrollView;\n    private int viewportWidth;\n    private String currentPath = \"{}\";\n    private String currentFragment = null;\n    private boolean dowePageTransitioning = false;\n    private boolean dowePageEntranceSuppressed = false;\n    private int dowePageTransitionSequence = 0;\n    private View dowePageContainerView = null;\n    private String doweMountedPath = null;\n    private String doweMountedLayout = null;\n    private boolean externalOpen = false;\n    private Runnable doweDrawerNavigationClose = null;
    private PopupWindow doweActiveOverlay = null;
    private int doweOverlayRender = 0;
    private int doweOverlayClaimed = 0;
    private final ArrayList<DoweRouteEntry> backStack = new ArrayList<>();\n    private final HashMap<String, Object> doweState = new HashMap<>();\n    private final HashMap<String, Object> doweInitial = new HashMap<>();\n    private final HashMap<String, Boolean> doweSideNavMemory = new HashMap<>();\n    private final HashMap<String, Boolean> doweTreeOpen = new HashMap<>();\n    private final HashMap<String, String> doweTreeSelected = new HashMap<>();\n    private final HashMap<String, String[]> doweSignalMetadata = new HashMap<>();\n    private final HashMap<String, Object> doweGlobalState = new HashMap<>();\n    private final HashMap<String, String> doweGlobalStorage = new HashMap<>();\n    private final HashMap<String, DoweAction> doweActions = new HashMap<>();\n    private final HashMap<String, DoweFormFieldMetadata[]> doweForms = new HashMap<>();\n    private final HashMap<String, View> sectionViews = new HashMap<>();\n    private final HashSet<String> doweLoaded = new HashSet<>();\n    private final HashSet<String> doweTouchedValidations = new HashSet<>();\n    private final HashSet<String> doweTouchedForms = new HashSet<>();\n\n",
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
    output.push_str(&dev_safe_area_color_methods(routes));
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
    output
}
