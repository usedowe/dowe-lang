r#"    private static final class DoweDeviceOption {
        final String profile;
        final DoweSvgView icon;

        DoweDeviceOption(String profile, DoweSvgView icon) {
            this.profile = profile;
            this.icon = icon;
        }
    }

    private FrameLayout doweDevice(String profile, String source, String title, boolean scripts, boolean autoplay, boolean hideControls, DoweDeviceOption[] options) {
        FrameLayout container = new FrameLayout(this);
        LinearLayout column = new LinearLayout(this);
        column.setOrientation(LinearLayout.VERTICAL);
        LinearLayout toolbar = new LinearLayout(this);
        toolbar.setGravity(Gravity.CENTER);
        FrameLayout stage = new FrameLayout(this);
        FrameLayout preview = doweIframe(source, title, scripts, autoplay);
        FrameLayout[] buttons = new FrameLayout[options.length];
        for (int index = 0; index < options.length; index++) {
            DoweDeviceOption option = options[index];
            FrameLayout button = new FrameLayout(this);
            button.setContentDescription(option.profile);
            button.setFocusable(true);
            button.setBackground(doweDeviceIconButtonBackground(option.profile.equals(profile)));
            button.setOnClickListener(target -> doweSetDeviceProfile(stage, preview, option.profile, options, buttons));
            option.icon.setImportantForAccessibility(View.IMPORTANT_FOR_ACCESSIBILITY_NO);
            button.addView(option.icon, new FrameLayout.LayoutParams(doweDp(24), doweDp(24), Gravity.CENTER));
            LinearLayout.LayoutParams buttonParams = new LinearLayout.LayoutParams(doweDp(40), doweDp(40));
            buttonParams.setMargins(doweDp(2), doweDp(4), doweDp(2), doweDp(4));
            if (!hideControls) toolbar.addView(button, buttonParams);
            buttons[index] = button;
        }
        stage.addView(preview);
        if (!hideControls) column.addView(toolbar, new LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT));
        column.addView(stage, new LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT));
        container.addView(column, new FrameLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT));
        stage.post(() -> doweSetDeviceProfile(stage, preview, profile, options, buttons));
        return container;
    }

    private GradientDrawable doweDeviceIconButtonBackground(boolean selected) {
        return doweInputBackground(selected ? DOWE_MUTED : DOWE_BACKGROUND, selected ? DOWE_PRIMARY : DOWE_BACKGROUND_TEXT, DOWE_RADIUS);
    }

    private void doweSetDeviceProfile(FrameLayout stage, FrameLayout preview, String profile, DoweDeviceOption[] options, FrameLayout[] buttons) {
        int width = "tablet".equals(profile) ? 768 : "laptop".equals(profile) ? 1440 : "monitor".equals(profile) ? 1920 : 390;
        int height = "tablet".equals(profile) ? 1024 : "laptop".equals(profile) ? 900 : "monitor".equals(profile) ? 1080 : 844;
        float zoom = Math.min(1f, stage.getWidth() / (float) doweDp(width));
        preview.setPivotX(doweDp(width) / 2f);
        preview.setPivotY(0f);
        preview.setScaleX(zoom);
        preview.setScaleY(zoom);
        preview.setLayoutParams(new FrameLayout.LayoutParams(doweDp(width), doweDp(height), Gravity.TOP | Gravity.CENTER_HORIZONTAL));
        stage.setLayoutParams(new LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, Math.round(doweDp(height) * zoom)));
        for (int index = 0; index < options.length; index++) {
            buttons[index].setBackground(doweDeviceIconButtonBackground(options[index].profile.equals(profile)));
            buttons[index].setSelected(options[index].profile.equals(profile));
            options[index].icon.setCurrentColor(options[index].profile.equals(profile) ? DOWE_PRIMARY : DOWE_BACKGROUND_TEXT);
        }
    }

    private FrameLayout doweIframe(String source, String title, boolean scripts, boolean autoplay) {
        FrameLayout container = new FrameLayout(this);
        container.setMinimumHeight(doweDp(192));
        WebView webView = new WebView(this);
        webView.setContentDescription(title);
        webView.getSettings().setJavaScriptEnabled(scripts);
        webView.getSettings().setDomStorageEnabled(true);
        webView.getSettings().setAllowFileAccess(false);
        webView.getSettings().setAllowContentAccess(false);
        webView.getSettings().setMediaPlaybackRequiresUserGesture(!autoplay);
        webView.getSettings().setSupportMultipleWindows(false);
        webView.setWebViewClient(new WebViewClient() {
            @Override
            public boolean shouldOverrideUrlLoading(WebView target, WebResourceRequest request) {
                return !doweIframeUrlAllowed(request.getUrl());
            }
        });
        String resolvedSource = doweIframeSource(source);
        if (resolvedSource != null) {
            webView.loadUrl(resolvedSource);
        }
        container.addView(webView, new FrameLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.MATCH_PARENT));
        return container;
    }

    private String doweIframeSource(String source) {
        if (source.startsWith("https://")) {
            return source;
        }
        if (!source.startsWith("/") || source.startsWith("//")) {
            return null;
        }
        String configured = DoweEnvironment.BACKEND_URL.replaceAll("/+$", "");
        String development = getSharedPreferences("dowe-hmr", 0).getString("endpoint", "");
        development = development == null ? "" : development.replaceAll("/+$", "");
        String base = doweIframeUrlAllowed(Uri.parse(development)) ? development : configured;
        if (!doweIframeUrlAllowed(Uri.parse(base))) {
            return null;
        }
        return java.net.URI.create(base).resolve(source).toString();
    }

    private boolean doweIframeUrlAllowed(Uri url) {
        if ("https".equals(url.getScheme())) {
            return true;
        }
        String host = url.getHost();
        return "http".equals(url.getScheme()) && ("localhost".equals(host) || "127.0.0.1".equals(host) || "::1".equals(host));
    }

    private float doweVideoAspect(String value) {
        if ("vertical".equals(value)) {
            return 9f / 16f;
        }
        if ("square".equals(value)) {
            return 1f;
        }
        return 16f / 9f;
    }

    private float doweImageAspect(String value) {
        if ("auto".equals(value)) {
            return 0f;
        }
        if (value != null) {
            try { return Math.max(0.01f, Float.parseFloat(value)); } catch (NumberFormatException ignored) {}
        }
        if ("vertical".equals(value)) {
            return 9f / 16f;
        }
        if ("square".equals(value)) {
            return 1f;
        }
        return 16f / 9f;
    }

"#
