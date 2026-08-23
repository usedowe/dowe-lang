fn dev_activity_code_and_forms_combo_layout() -> &'static str {
    r##"    private void dowePinAppBar(ViewGroup parent, ViewGroup appBar, boolean dockOnScroll, int surfaceColor) {
        ViewGroup background = (ViewGroup) scrollView.getParent();
        View previous = background.findViewWithTag("dowe-pinned-appbar");
        if (previous != null) {
            background.removeView(previous);
        }
        View previousSafeArea = background.findViewWithTag("dowe-pinned-appbar-safe-area");
        if (previousSafeArea != null) {
            background.removeView(previousSafeArea);
        }
        int appBarWidth = Math.max(0, getResources().getDisplayMetrics().widthPixels - scrollView.getPaddingLeft() - scrollView.getPaddingRight());
        int childWidthSpec = View.MeasureSpec.makeMeasureSpec(appBarWidth, View.MeasureSpec.AT_MOST);
        int appBarHeight = doweDp(48);
        for (int index = 0; index < appBar.getChildCount(); index++) {
            View child = appBar.getChildAt(index);
            child.measure(childWidthSpec, View.MeasureSpec.makeMeasureSpec(0, View.MeasureSpec.UNSPECIFIED));
            appBarHeight = Math.max(appBarHeight, child.getMeasuredHeight());
        }
        dowePinnedAppBarDockOnScroll = dockOnScroll;
        dowePinnedAppBarColor = surfaceColor;
        dowePinnedAppBarHeight = appBarHeight;
        dowePinnedAppBarDockProgress = dockOnScroll && scrollView.getScrollY() > doweDp(100) ? 1f : 0f;
        View placeholder = new View(this);
        int outerVertical = dockOnScroll ? Math.round(doweDp(8) * (1f - dowePinnedAppBarDockProgress)) : 0;
        placeholder.setLayoutParams(new LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, appBarHeight + outerVertical * 2));
        doweAdd(parent, placeholder);
        dowePinnedAppBarPlaceholder = placeholder;
        View safeArea = new View(this);
        safeArea.setTag("dowe-pinned-appbar-safe-area");
        safeArea.setBackgroundColor(DOWE_BACKGROUND);
        FrameLayout.LayoutParams safeAreaParams = new FrameLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, scrollView.getPaddingTop(), Gravity.TOP | Gravity.START);
        background.addView(safeArea, safeAreaParams);
        View bottomSafeArea = new View(this);
        bottomSafeArea.setTag("dowe-pinned-appbar-bottom-safe-area");
        bottomSafeArea.setBackgroundColor(DOWE_BACKGROUND);
        FrameLayout.LayoutParams bottomSafeAreaParams = new FrameLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, scrollView.getPaddingBottom(), Gravity.BOTTOM | Gravity.START);
        background.addView(bottomSafeArea, bottomSafeAreaParams);
        appBar.setTag("dowe-pinned-appbar");
        FrameLayout.LayoutParams params = new FrameLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, appBarHeight, Gravity.TOP | Gravity.START);
        params.setMargins(scrollView.getPaddingLeft(), scrollView.getPaddingTop(), scrollView.getPaddingRight(), 0);
        background.addView(appBar, params);
        View divider = new View(this);
        divider.setTag("dowe-pinned-appbar-divider");
        divider.setBackgroundColor(DOWE_MUTED);
        divider.setAlpha(dockOnScroll ? dowePinnedAppBarDockProgress : 0f);
        background.addView(divider, new FrameLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, doweDp(1), Gravity.TOP | Gravity.START));
        dowePinnedAppBarDivider = divider;
        scrollView.post(this::doweRelayoutPinnedAppBar);
    }

    private void doweUpdatePinnedAppBarDock(boolean docked, boolean animate) {
        if (!dowePinnedAppBarDockOnScroll) {
            return;
        }
        float target = docked ? 1f : 0f;
        if (Math.abs(target - dowePinnedAppBarDockProgress) < 0.001f) {
            return;
        }
        if (dowePinnedAppBarAnimator != null) {
            dowePinnedAppBarAnimator.cancel();
        }
        boolean animationsEnabled = Build.VERSION.SDK_INT < 26 || ValueAnimator.areAnimatorsEnabled();
        if (!animate || !animationsEnabled) {
            doweApplyPinnedAppBarDockProgress(target);
            return;
        }
        dowePinnedAppBarAnimator = ValueAnimator.ofFloat(dowePinnedAppBarDockProgress, target);
        dowePinnedAppBarAnimator.setDuration(300);
        dowePinnedAppBarAnimator.setInterpolator(new PathInterpolator(0.4f, 0f, 0.2f, 1f));
        dowePinnedAppBarAnimator.addUpdateListener(value -> doweApplyPinnedAppBarDockProgress((float) value.getAnimatedValue()));
        dowePinnedAppBarAnimator.start();
    }

    private void doweApplyPinnedAppBarDockProgress(float progress) {
        dowePinnedAppBarDockProgress = progress;
        if (scrollView == null || scrollView.getParent() == null) {
            return;
        }
        ViewGroup background = (ViewGroup) scrollView.getParent();
        View appBar = background.findViewWithTag("dowe-pinned-appbar");
        if (appBar == null) {
            return;
        }
        int horizontal = Math.round(doweDp(16) * (1f - progress));
        int vertical = Math.round(doweDp(8) * (1f - progress));
        int leftInset = scrollView.getPaddingLeft();
        int topInset = scrollView.getPaddingTop();
        int rightInset = scrollView.getPaddingRight();
        if (appBar.getLayoutParams() instanceof FrameLayout.LayoutParams) {
            FrameLayout.LayoutParams params = (FrameLayout.LayoutParams) appBar.getLayoutParams();
            params.setMargins(leftInset + horizontal, topInset + vertical, rightInset + horizontal, 0);
            appBar.setLayoutParams(params);
        }
        appBar.setBackground(doweStyledBackground(dowePinnedAppBarColor, doweAlpha(DOWE_MUTED, 1f - progress), 1, DOWE_RADIUS * (1f - progress)));
        if (dowePinnedAppBarPlaceholder != null && dowePinnedAppBarPlaceholder.getLayoutParams() instanceof LinearLayout.LayoutParams) {
            LinearLayout.LayoutParams placeholderParams = (LinearLayout.LayoutParams) dowePinnedAppBarPlaceholder.getLayoutParams();
            placeholderParams.height = dowePinnedAppBarHeight + vertical * 2;
            dowePinnedAppBarPlaceholder.setLayoutParams(placeholderParams);
        }
        if (dowePinnedAppBarDivider != null && dowePinnedAppBarDivider.getLayoutParams() instanceof FrameLayout.LayoutParams) {
            FrameLayout.LayoutParams dividerParams = (FrameLayout.LayoutParams) dowePinnedAppBarDivider.getLayoutParams();
            dividerParams.setMargins(leftInset + horizontal, topInset + vertical + dowePinnedAppBarHeight - doweDp(1), rightInset + horizontal, 0);
            dowePinnedAppBarDivider.setLayoutParams(dividerParams);
            dowePinnedAppBarDivider.setAlpha(progress);
        }
    }

    private void doweRelayoutPinnedAppBar() {
        if (scrollView == null || scrollView.getParent() == null) {
            return;
        }
        ViewGroup background = (ViewGroup) scrollView.getParent();
        int leftInset = scrollView.getPaddingLeft();
        int topInset = scrollView.getPaddingTop();
        int rightInset = scrollView.getPaddingRight();
        int bottomInset = scrollView.getPaddingBottom();
        WindowInsets rootInsets = background.getRootWindowInsets();
        if (rootInsets != null) {
            if (Build.VERSION.SDK_INT >= 30) {
                Insets safe = rootInsets.getInsets(WindowInsets.Type.systemBars() | WindowInsets.Type.displayCutout());
                leftInset = safe.left;
                topInset = safe.top;
                rightInset = safe.right;
                bottomInset = safe.bottom;
            } else {
                leftInset = rootInsets.getSystemWindowInsetLeft();
                topInset = rootInsets.getSystemWindowInsetTop();
                rightInset = rootInsets.getSystemWindowInsetRight();
                bottomInset = rootInsets.getSystemWindowInsetBottom();
            }
        }
        scrollView.setPadding(leftInset, topInset, rightInset, bottomInset);
        View appBar = background.findViewWithTag("dowe-pinned-appbar");
        if (dowePinnedAppBarDockOnScroll) {
            doweApplyPinnedAppBarDockProgress(dowePinnedAppBarDockProgress);
        } else if (appBar != null && appBar.getLayoutParams() instanceof FrameLayout.LayoutParams) {
            FrameLayout.LayoutParams appBarParams = (FrameLayout.LayoutParams) appBar.getLayoutParams();
            appBarParams.setMargins(leftInset, topInset, rightInset, 0);
            appBar.setLayoutParams(appBarParams);
        }
        View safeArea = background.findViewWithTag("dowe-pinned-appbar-safe-area");
        if (safeArea != null && safeArea.getLayoutParams() instanceof FrameLayout.LayoutParams) {
            FrameLayout.LayoutParams safeAreaParams = (FrameLayout.LayoutParams) safeArea.getLayoutParams();
            safeAreaParams.height = topInset;
            safeArea.setLayoutParams(safeAreaParams);
        }
        View bottomSafeArea = background.findViewWithTag("dowe-pinned-appbar-bottom-safe-area");
        if (bottomSafeArea != null && bottomSafeArea.getLayoutParams() instanceof FrameLayout.LayoutParams) {
            FrameLayout.LayoutParams bottomSafeAreaParams = (FrameLayout.LayoutParams) bottomSafeArea.getLayoutParams();
            bottomSafeAreaParams.height = bottomInset;
            bottomSafeArea.setLayoutParams(bottomSafeAreaParams);
        }
    }

    private LinearLayout.LayoutParams doweLinearLayoutParams(ViewGroup.LayoutParams current) {
        LinearLayout.LayoutParams params;
        if (current instanceof LinearLayout.LayoutParams) {
            params = new LinearLayout.LayoutParams((LinearLayout.LayoutParams) current);
        } else if (current != null) {
            params = new LinearLayout.LayoutParams(current.width, current.height);
        } else {
            params = new LinearLayout.LayoutParams(ViewGroup.LayoutParams.WRAP_CONTENT, ViewGroup.LayoutParams.WRAP_CONTENT);
        }
        doweCopyMargins(params, current);
        return params;
    }

    private FrameLayout.LayoutParams doweFrameLayoutParams(ViewGroup.LayoutParams current) {
        FrameLayout.LayoutParams params;
        if (current instanceof FrameLayout.LayoutParams) {
            params = new FrameLayout.LayoutParams((FrameLayout.LayoutParams) current);
        } else if (current != null) {
            params = new FrameLayout.LayoutParams(current.width, current.height);
        } else {
            params = new FrameLayout.LayoutParams(ViewGroup.LayoutParams.WRAP_CONTENT, ViewGroup.LayoutParams.WRAP_CONTENT);
        }
        doweCopyMargins(params, current);
        return params;
    }

    private ScrollView.LayoutParams doweScrollViewLayoutParams(ViewGroup.LayoutParams current) {
        int width = current != null ? current.width : ViewGroup.LayoutParams.MATCH_PARENT;
        int height = current != null ? current.height : ViewGroup.LayoutParams.WRAP_CONTENT;
        ScrollView.LayoutParams params = new ScrollView.LayoutParams(width, height);
        doweCopyMargins(params, current);
        return params;
    }

    private HorizontalScrollView.LayoutParams doweHorizontalScrollViewLayoutParams(ViewGroup.LayoutParams current) {
        int width = current != null ? current.width : ViewGroup.LayoutParams.WRAP_CONTENT;
        int height = current != null ? current.height : ViewGroup.LayoutParams.WRAP_CONTENT;
        HorizontalScrollView.LayoutParams params = new HorizontalScrollView.LayoutParams(width, height);
        doweCopyMargins(params, current);
        return params;
    }

    private void doweCopyMargins(ViewGroup.MarginLayoutParams target, ViewGroup.LayoutParams source) {
        if (source instanceof ViewGroup.MarginLayoutParams) {
            ViewGroup.MarginLayoutParams margins = (ViewGroup.MarginLayoutParams) source;
            target.setMargins(margins.leftMargin, margins.topMargin, margins.rightMargin, margins.bottomMargin);
        }
    }

"##
}
