r#"        if (id != null) {
            sectionViews.put(id, view);
        }
    }

    private void doweScrollToFragment() {
        if (currentFragment == null || scrollView == null) {
            return;
        }
        View laidOutTarget = sectionViews.get(currentFragment);
        if (laidOutTarget != null && laidOutTarget.isLaidOut()) {
            laidOutTarget.post(() -> doweRevealSection(laidOutTarget));
            return;
        }
        root.getViewTreeObserver().addOnPreDrawListener(new android.view.ViewTreeObserver.OnPreDrawListener() {
            @Override
            public boolean onPreDraw() {
                if (root.getViewTreeObserver().isAlive()) {
                    root.getViewTreeObserver().removeOnPreDrawListener(this);
                }
                View target = sectionViews.get(currentFragment);
                if (target != null) {
                    doweRevealSection(target);
                }
                return true;
            }
        });
    }

    private void doweRevealSection(View target) {
        int[] targetLocation = new int[2];
        int[] scrollLocation = new int[2];
        target.getLocationInWindow(targetLocation);
        scrollView.getLocationInWindow(scrollLocation);
        int visibleTop = scrollLocation[1] + scrollView.getPaddingTop();
        View pinnedAppBar = ((ViewGroup) scrollView.getParent()).findViewWithTag("dowe-pinned-appbar");
        if (pinnedAppBar != null) {
            int[] appBarLocation = new int[2];
            pinnedAppBar.getLocationInWindow(appBarLocation);
            visibleTop = Math.max(visibleTop, appBarLocation[1] + pinnedAppBar.getHeight());
        }
        int destination = Math.max(0, scrollView.getScrollY() + targetLocation[1] - visibleTop);
        scrollView.smoothScrollTo(0, destination);
    }

    private void doweAnimate(View view, String preset) {
        if (preset == null || "none".equals(preset)) {
            return;
        }
        if (dowePageEntranceSuppressed) {
            view.setAlpha(1f);
            return;
        }
        float baseTranslationX = view.getTranslationX();
        float baseTranslationY = view.getTranslationY();
        float baseScaleX = view.getScaleX();
        float baseScaleY = view.getScaleY();
        view.setAlpha(0f);
        if ("slideUp".equals(preset)) {
            view.setTranslationY(baseTranslationY + doweDp(16));
        } else if ("slideDown".equals(preset)) {
            view.setTranslationY(baseTranslationY - doweDp(16));
        } else if ("slideLeft".equals(preset)) {
            view.setTranslationX(baseTranslationX + doweDp(16));
        } else if ("slideRight".equals(preset)) {
            view.setTranslationX(baseTranslationX - doweDp(16));
        } else if ("scaleIn".equals(preset)) {
            view.setScaleX(baseScaleX * 0.96f);
            view.setScaleY(baseScaleY * 0.96f);
        }
        view.animate().alpha(1f).translationX(baseTranslationX).translationY(baseTranslationY).scaleX(baseScaleX).scaleY(baseScaleY).setDuration(220).start();
    }

    private void doweGesture(View view, String preset, String transition) {
        float baseTranslationY = view.getTranslationY();
        float baseScaleX = view.getScaleX();
        float baseScaleY = view.getScaleY();
        float baseRotation = view.getRotation();
        long duration = "none".equals(transition) ? 0L : "quick".equals(transition) ? 120L : "spring".equals(transition) ? 320L : 220L;
        float pressedTranslationY = "lift".equals(preset) ? baseTranslationY - doweDp(4) : baseTranslationY;
        float pressedScaleX = "press".equals(preset) ? baseScaleX * 0.94f : "lift".equals(preset) ? baseScaleX * 0.98f : "grow".equals(preset) ? baseScaleX * 1.04f : baseScaleX;
        float pressedScaleY = "press".equals(preset) ? baseScaleY * 0.94f : "lift".equals(preset) ? baseScaleY * 0.98f : "grow".equals(preset) ? baseScaleY * 1.04f : baseScaleY;
        float pressedRotation = "tilt".equals(preset) ? baseRotation + 3f : baseRotation;
        android.animation.AnimatorSet pressedAnimator = new android.animation.AnimatorSet();
        pressedAnimator.playTogether(
            android.animation.ObjectAnimator.ofFloat(view, android.view.View.TRANSLATION_Y, pressedTranslationY),
            android.animation.ObjectAnimator.ofFloat(view, android.view.View.SCALE_X, pressedScaleX),
            android.animation.ObjectAnimator.ofFloat(view, android.view.View.SCALE_Y, pressedScaleY),
            android.animation.ObjectAnimator.ofFloat(view, android.view.View.ROTATION, pressedRotation)
        );
        pressedAnimator.setDuration(duration);
        android.animation.AnimatorSet releasedAnimator = new android.animation.AnimatorSet();
        releasedAnimator.playTogether(
            android.animation.ObjectAnimator.ofFloat(view, android.view.View.TRANSLATION_Y, baseTranslationY),
            android.animation.ObjectAnimator.ofFloat(view, android.view.View.SCALE_X, baseScaleX),
            android.animation.ObjectAnimator.ofFloat(view, android.view.View.SCALE_Y, baseScaleY),
            android.animation.ObjectAnimator.ofFloat(view, android.view.View.ROTATION, baseRotation)
        );
        releasedAnimator.setDuration(duration);
        android.animation.StateListAnimator stateAnimator = new android.animation.StateListAnimator();
        stateAnimator.addState(new int[]{android.R.attr.state_pressed}, pressedAnimator);
        stateAnimator.addState(new int[]{}, releasedAnimator);
        DOWE_GESTURE_ANIMATORS.put(view, stateAnimator);
        view.setStateListAnimator(stateAnimator);
    }

    private boolean doweSideNavExpanded(String key, boolean initial) {
        Boolean expanded = doweSideNavMemory.get(key);
        if (expanded == null) {
            doweSideNavMemory.put(key, initial);
            return initial;
        }
        return expanded;
    }

    private void doweToggleSideNavSubmenu(View view, View arrow, String key) {
        view.animate().withEndAction(null).cancel();
        if (view.getVisibility() == View.VISIBLE) {
            doweSideNavMemory.put(key, false);
            if (arrow != null) {
                arrow.animate().rotation(0f).setDuration(140).start();
            }
            view.animate().alpha(0f).translationY(-doweDp(4)).setDuration(140).withEndAction(() -> {
                view.setVisibility(View.GONE);
                view.setAlpha(1f);
                view.setTranslationY(0f);
            }).start();
            return;
        }
        doweSideNavMemory.put(key, true);
        if (arrow != null) {
            arrow.animate().rotation(90f).setDuration(160).start();
        }
        view.setAlpha(0f);
        view.setTranslationY(-doweDp(4));
        view.setVisibility(View.VISIBLE);
        view.animate().alpha(1f).translationY(0f).setDuration(160).withEndAction(null).start();
    }

    private static final class DoweSideNavEntry {
        final String id;
        final String kind;
        final String label;
        final String description;
        final String status;
        final String operation;
        final String path;
        final String fragment;
        final boolean open;
        final boolean bordered;
        final ArrayList<DoweSideNavEntry> children;

        DoweSideNavEntry(String id, String kind, String label, String description, String status, String operation, String path, String fragment, boolean open, boolean bordered, ArrayList<DoweSideNavEntry> children) {
            this.id = id;
            this.kind = kind;
            this.label = label;
            this.description = description;
            this.status = status;
            this.operation = operation;
            this.path = path;
            this.fragment = fragment;
            this.open = open;
            this.bordered = bordered;
            this.children = children == null ? new ArrayList<>() : children;
        }
    }

    private void doweRenderSideNav(LinearLayout parent, ArrayList<DoweSideNavEntry> entries, String stateKey, boolean wide, int paddingHorizontal, int paddingVertical, int gap, int labelSize, int descriptionSize, int backgroundColor, int activeContentColor, int titleColor, String font) {
        if (wide) parent.setLayoutParams(new LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT));
        for (DoweSideNavEntry entry : entries) {
            if ("divider".equals(entry.kind)) {
                View divider = new View(this);
                divider.setBackgroundColor(DOWE_MUTED);
                divider.setLayoutParams(new LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, doweDp(1)));
                doweAdd(parent, divider, 8, false);
            } else if ("submenu".equals(entry.kind)) {
                String submenuKey = stateKey + ":" + entry.id;
                boolean expanded = doweSideNavExpanded(submenuKey, entry.open);
                LinearLayout trigger = doweSideNavRow(entry, false, wide, paddingHorizontal, paddingVertical, gap, labelSize, descriptionSize, backgroundColor, activeContentColor, titleColor, font, null, expanded);
                doweAdd(parent, trigger);
                LinearLayout submenu = doweContainer(entry.bordered);
                submenu.setPadding(doweDp(16), 0, 0, 0);
                submenu.setVisibility(expanded ? View.VISIBLE : View.GONE);
                doweAdd(parent, submenu);
                LinearLayout submenuContent = doweSideNavSubmenuContent(submenu, entry.bordered);
                View arrow = (View) trigger.getTag();
                trigger.setOnClickListener(v -> doweToggleSideNavSubmenu(submenu, arrow, submenuKey));
                doweRenderSideNav(submenuContent, entry.children, stateKey, wide, paddingHorizontal, paddingVertical, gap, labelSize, descriptionSize, backgroundColor, activeContentColor, titleColor, font);
            } else {
                LinearLayout row = doweSideNavRow(entry, "header".equals(entry.kind), wide, paddingHorizontal, paddingVertical, gap, labelSize, descriptionSize, backgroundColor, activeContentColor, titleColor, font, doweSideNavAction(entry), null);
                doweAdd(parent, row);
            }
        }
    }

    private LinearLayout doweSideNavSubmenuContent(LinearLayout submenu, boolean bordered) {
        if (!bordered) {
            return submenu;
        }
        View border = new View(this);
        border.setBackgroundColor(DOWE_MUTED);
        border.setLayoutParams(new LinearLayout.LayoutParams(doweDp(1), ViewGroup.LayoutParams.MATCH_PARENT));
        doweAdd(submenu, border);
        LinearLayout content = doweContainer(false);
        content.setPadding(doweDp(8), 0, 0, 0);
        doweAdd(submenu, content);
        return content;
    }

    private LinearLayout doweSideNavRow(DoweSideNavEntry entry, boolean header, boolean wide, int paddingHorizontal, int paddingVertical, int gap, int labelSize, int descriptionSize, int backgroundColor, int activeContentColor, int titleColor, String font, Runnable action, Boolean submenuOpen) {
        LinearLayout view = doweContainer(true);
        view.setLayoutParams(new LinearLayout.LayoutParams(wide ? ViewGroup.LayoutParams.MATCH_PARENT : ViewGroup.LayoutParams.WRAP_CONTENT, ViewGroup.LayoutParams.WRAP_CONTENT));
        view.setGravity(Gravity.CENTER_VERTICAL);
        view.setPadding(doweDp(paddingHorizontal), doweDp(paddingVertical), doweDp(paddingHorizontal), doweDp(paddingVertical));
        boolean active = entry.path != null && entry.path.equals(currentPath);
        if (active) {
            view.setBackground(doweBackground(backgroundColor, DOWE_RADIUS));
        }
        int rowContentColor = active ? activeContentColor : DOWE_BACKGROUND_TEXT;
        LinearLayout copy = doweContainer(false);
        copy.setLayoutParams(new LinearLayout.LayoutParams(0, ViewGroup.LayoutParams.WRAP_CONTENT, 1f));
        doweAdd(view, copy);
        TextView label = doweText(entry.label, header ? titleColor : rowContentColor, labelSize, header ? 600 : 400, 0f, labelSize, font);
        doweAdd(copy, label);
        if (entry.description != null) {
            TextView description = doweText(entry.description, rowContentColor, descriptionSize, 400, 0f, descriptionSize, font);
            description.setAlpha(0.72f);
            doweAdd(copy, description);
        }
        if (entry.status != null) {
            TextView status = doweSideNavStatus(entry.status, descriptionSize, font);
            doweAdd(view, status, gap, true);
        }
        if (submenuOpen != null) {
            DoweSvgView arrow = doweSideNavArrow(rowContentColor);
            arrow.setRotation(submenuOpen ? 90f : 0f);
            view.setTag(arrow);
            doweAdd(view, arrow, gap, true);
        }
        if (action != null) {
            view.setOnClickListener(v -> action.run());
        }
        return view;
    }

    private TextView doweSideNavStatus(String text, float descriptionSize, String font) {
        TextView status = doweText(text, DOWE_MUTED_TEXT, descriptionSize, 600, 0f, descriptionSize, font);
        status.setPadding(doweDp(8), doweDp(2), doweDp(8), doweDp(2));
        status.setBackground(doweBackground(DOWE_MUTED, 999f));
        return status;
    }

    private DoweSvgView doweSideNavArrow(int color) {
        ArrayList<DoweSvgPathEntry> paths = new ArrayList<>();
        paths.add(new DoweSvgPathEntry("M0 0h24v24H0z", false, null));
        paths.add(new DoweSvgPathEntry("__DOWE_SIDE_NAV_SUBMENU_ARROW_PATH__", true, null));
        DoweSvgView view = new DoweSvgView(this, 0f, 0f, 24f, 24f, color, paths);
        view.setLayoutParams(new LinearLayout.LayoutParams(doweDp(16), doweDp(16)));
        return view;
    }

    private DoweSvgView doweNavMenuArrow(int color) {
        DoweSvgView view = doweSideNavArrow(color);
        view.setRotation(90f);
        return view;
    }

    private Runnable doweSideNavAction(DoweSideNavEntry entry) {
        if (entry.path == null) {
            return null;
        }
        return () -> doweNavigate(entry.operation == null ? "push" : entry.operation, entry.path, entry.fragment);
    }

"#
