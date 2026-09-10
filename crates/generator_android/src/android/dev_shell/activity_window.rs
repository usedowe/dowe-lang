fn append_dev_activity_window(output: &mut String) {
    output.push_str(
        r#"    private Window getWindow() {
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

    private float doweColorLuminance(int color) {
        float red = Color.red(color) / 255f;
        float green = Color.green(color) / 255f;
        float blue = Color.blue(color) / 255f;
        red = red <= 0.03928f ? red / 12.92f : (float) Math.pow((red + 0.055f) / 1.055f, 2.4);
        green = green <= 0.03928f ? green / 12.92f : (float) Math.pow((green + 0.055f) / 1.055f, 2.4);
        blue = blue <= 0.03928f ? blue / 12.92f : (float) Math.pow((blue + 0.055f) / 1.055f, 2.4);
        return 0.2126f * red + 0.7152f * green + 0.0722f * blue;
    }

    private void doweApplySystemBarAppearance() {
        boolean useDarkStatusIcons = doweColorLuminance(doweSafeAreaTopColor) > 0.179f;
        boolean useDarkNavigationIcons = doweColorLuminance(doweSafeAreaBottomColor) > 0.179f;
        if (Build.VERSION.SDK_INT >= 30) {
            int statusMask = android.view.WindowInsetsController.APPEARANCE_LIGHT_STATUS_BARS;
            int navigationMask = android.view.WindowInsetsController.APPEARANCE_LIGHT_NAVIGATION_BARS;
            int appearance = (useDarkStatusIcons ? statusMask : 0) |
                (useDarkNavigationIcons ? navigationMask : 0);
            getWindow().getInsetsController().setSystemBarsAppearance(appearance, statusMask | navigationMask);
        } else {
            int visibility = View.SYSTEM_UI_FLAG_LAYOUT_STABLE |
                View.SYSTEM_UI_FLAG_LAYOUT_FULLSCREEN |
                View.SYSTEM_UI_FLAG_LAYOUT_HIDE_NAVIGATION;
            if (useDarkStatusIcons) visibility |= View.SYSTEM_UI_FLAG_LIGHT_STATUS_BAR;
            if (useDarkNavigationIcons) visibility |= View.SYSTEM_UI_FLAG_LIGHT_NAVIGATION_BAR;
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

    private ViewGroup doweCreatePageContainer(ViewGroup parent) {
        LinearLayout page = doweContainer(false);
        page.setLayoutParams(new LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT));
        page.setAlpha(dowePageTransitioning ? 0f : 1f);
        doweAdd(parent, page);
        dowePageContainerView = page;
        return page;
    }

    private void doweStartPageTransition() {
        dowePageTransitionSequence++;
        dowePageEntranceSuppressed = true;
        dowePageTransitioning = ValueAnimator.areAnimatorsEnabled();
        if (root == null) {
            return;
        }
        if (dowePageContainerView != null) {
            dowePageContainerView.animate().cancel();
        }
        root.animate().cancel();
    }

    private void doweFinishPageTransition() {
        View target = dowePageContainerView == null ? root : dowePageContainerView;
        if (target == null) {
            return;
        }
        int sequence = dowePageTransitionSequence;
        target.post(() -> {
            if (sequence != dowePageTransitionSequence || !target.isAttachedToWindow()) {
                return;
            }
            if (!dowePageTransitioning || !ValueAnimator.areAnimatorsEnabled()) {
                target.setAlpha(1f);
                dowePageTransitioning = false;
                return;
            }
            target.animate()
                .alpha(1f)
                .setDuration(__DOWE_PAGE_TRANSITION_DURATION_MS__)
                .setInterpolator(new PathInterpolator(__DOWE_PAGE_TRANSITION_X1__f, __DOWE_PAGE_TRANSITION_Y1__f, __DOWE_PAGE_TRANSITION_X2__f, __DOWE_PAGE_TRANSITION_Y2__f))
                .withEndAction(() -> {
                    if (sequence == dowePageTransitionSequence) {
                        dowePageTransitioning = false;
                    }
                })
                .start();
        });
    }

"#,
    );
}
