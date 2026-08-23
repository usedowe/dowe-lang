        doweRunAction(id, item, () -> {
            if (doweActiveOverlay != null && doweActiveOverlay.isShowing()) {
                doweActiveOverlay.dismiss();
            }
            renderCurrentRoute(false);
        });
    }

    private void doweRunAction(String id, Map<String, Object> item, Runnable completion) {
        DoweAction action = doweActions.get(id);
        if (action == null) {
            completion.run();
            return;
        }
        if ("assign".equals(action.kind)) {
            doweWrite(action.target, action.stdlibNamespace == null ? doweSetValue(action.source, item) : doweStdlib(action, item));
            completion.run();
            return;
        }
        if ("reset".equals(action.kind)) {
            doweWrite(action.target, doweInitial.get(action.target));
            completion.run();
            return;
        }
        if ("sequence".equals(action.kind)) {
            doweRunSteps(action.steps, 0, item, new HashMap<>(), completion);
            return;
        }
        doweRequest(action, item, (successful, responseData) -> {
            if (successful) {
                if (action.update != null) doweWrite(action.update, responseData);
                if (action.reset != null) doweWrite(action.reset, doweInitial.get(action.reset));
                doweSetAlert(action.successAlert, "success", action.successMessage == null ? "Request completed" : action.successMessage);
            } else {
                doweSetAlert(action.errorAlert, "error", action.errorMessage == null ? "Request failed" : action.errorMessage);
            }
            completion.run();
        });
    }

    private void doweRunSteps(DoweStep[] steps, int index, Map<String, Object> item, HashMap<String, Object> results, Runnable completion) {
        if (steps == null || index >= steps.length) {
            completion.run();
            return;
        }
        DoweStep step = steps[index];
        if ("validate".equals(step.kind)) {
            if (!doweValidateForm(step.target)) {
                renderCurrentRoute(false);
                return;
            }
            doweRunSteps(steps, index + 1, item, results, completion);
            return;
        }
        if ("request".equals(step.kind)) {
            doweRequest(step.request, item, (ok, data) -> {
                results.put(step.result, doweObject("ok", ok, "data", data));
                doweRunSteps(steps, index + 1, item, results, completion);
            });
            return;
        }
        if ("branch".equals(step.kind)) {
            Object result = results.get(step.result);
            boolean ok = result instanceof Map && Boolean.TRUE.equals(((Map<?, ?>) result).get("ok"));
            doweRunSteps(ok ? step.success : step.error, 0, item, results, () -> doweRunSteps(steps, index + 1, item, results, completion));
            return;
        }
        if ("redirect".equals(step.kind)) {
            doweNavigate("replace", step.target, null);
            return;
        }
        if ("assign".equals(step.kind)) {
            Object value = step.hasLiteral ? step.literal : step.call == null ? doweSetValue(step.source, item, results) : doweStdlib(step.call, item);
            doweWrite(step.target, value);
        } else if ("reset".equals(step.kind)) {
            doweWrite(step.target, doweInitial.get(step.target));
        } else {
            doweShowToast(step);
        }
        doweRunSteps(steps, index + 1, item, results, completion);
    }

    private int doweToastFamily(String scheme) {
        if ("background".equals(scheme)) return DOWE_BACKGROUND;
        if ("surface".equals(scheme)) return DOWE_SURFACE;
        if ("secondary".equals(scheme)) return DOWE_SECONDARY;
        if ("accent".equals(scheme)) return DOWE_ACCENT;
        if ("muted".equals(scheme)) return DOWE_MUTED;
        if ("success".equals(scheme)) return DOWE_SUCCESS;
        if ("info".equals(scheme)) return DOWE_INFO;
        if ("warning".equals(scheme)) return DOWE_WARNING;
        if ("danger".equals(scheme)) return DOWE_DANGER;
        return DOWE_PRIMARY;
    }

    private int doweToastTextFamily(String scheme) {
        if ("background".equals(scheme)) return DOWE_BACKGROUND_TEXT;
        if ("surface".equals(scheme)) return DOWE_SURFACE_TEXT;
        if ("secondary".equals(scheme)) return DOWE_SECONDARY_TEXT;
        if ("accent".equals(scheme)) return DOWE_ACCENT_TEXT;
        if ("muted".equals(scheme)) return DOWE_MUTED_TEXT;
        if ("success".equals(scheme)) return DOWE_SUCCESS_TEXT;
        if ("info".equals(scheme)) return DOWE_INFO_TEXT;
        if ("warning".equals(scheme)) return DOWE_WARNING_TEXT;
        if ("danger".equals(scheme)) return DOWE_DANGER_TEXT;
        return DOWE_PRIMARY_TEXT;
    }

    private int doweToastSoftFamily(String scheme) {
        if ("background".equals(scheme)) return DOWE_BACKGROUND;
        if ("surface".equals(scheme)) return DOWE_SURFACE;
        if ("secondary".equals(scheme)) return DOWE_SECONDARY;
        if ("accent".equals(scheme)) return DOWE_ACCENT;
        if ("muted".equals(scheme)) return DOWE_MUTED;
        if ("success".equals(scheme)) return DOWE_SUCCESS;
        if ("info".equals(scheme)) return DOWE_INFO;
        if ("warning".equals(scheme)) return DOWE_WARNING;
        if ("danger".equals(scheme)) return DOWE_DANGER;
        return DOWE_PRIMARY;
    }

    private int doweToastSoftTextFamily(String scheme) {
        if ("background".equals(scheme)) return DOWE_BACKGROUND_TEXT;
        if ("surface".equals(scheme)) return DOWE_SURFACE_TEXT;
        if ("secondary".equals(scheme)) return DOWE_SECONDARY_TEXT;
        if ("accent".equals(scheme)) return DOWE_ACCENT_TEXT;
        if ("muted".equals(scheme)) return DOWE_MUTED_TEXT;
        if ("success".equals(scheme)) return DOWE_SUCCESS_TEXT;
        if ("info".equals(scheme)) return DOWE_INFO_TEXT;
        if ("warning".equals(scheme)) return DOWE_WARNING_TEXT;
        if ("danger".equals(scheme)) return DOWE_DANGER_TEXT;
        return DOWE_PRIMARY_TEXT;
    }

    private void doweShowToast(DoweStep step) {
        String scheme = step.scheme == null ? ("error".equals(step.kind) ? "danger" : step.kind) : step.scheme;
        String variant = step.variant == null ? "solid" : step.variant;
        String position = step.position == null ? "top-right" : step.position;
        int background = "soft".equals(variant) ? doweToastSoftFamily(scheme) : "outlined".equals(variant) ? ("background".equals(scheme) ? DOWE_BACKGROUND : DOWE_SURFACE) : "ghost".equals(variant) ? Color.TRANSPARENT : doweToastFamily(scheme);
        int content = "solid".equals(variant) ? doweToastTextFamily(scheme) : "soft".equals(variant) ? doweToastSoftTextFamily(scheme) : "outlined".equals(variant) ? ("background".equals(scheme) ? DOWE_BACKGROUND_TEXT : DOWE_SURFACE_TEXT) : ("background".equals(scheme) || "surface".equals(scheme) ? doweToastTextFamily(scheme) : doweToastFamily(scheme));
        Integer border = "outlined".equals(variant) ? doweToastFamily(scheme) : null;
        LinearLayout panel = doweContainer(true);
        panel.setGravity(Gravity.CENTER_VERTICAL);
        panel.setPadding(doweDp(16), doweDp(12), doweDp(12), doweDp(12));
        panel.setBackground(doweInputBackground(background, border, DOWE_RADIUS));
        String text = (step.title == null || step.title.isEmpty() ? "" : step.title + "\n") + (step.message == null ? "" : step.message);
        TextView message = doweText(text, content, 14f, 500, 0f, 1.25f, null);
        message.setLayoutParams(new LinearLayout.LayoutParams(0, ViewGroup.LayoutParams.WRAP_CONTENT, 1f));
        doweAdd(panel, message);
        FrameLayout close = new FrameLayout(this);
        close.setBackground(doweBackground(DOWE_MUTED, 999f));
        close.setContentDescription("Close toast");
        close.setFocusable(true);
        close.setLayoutParams(new LinearLayout.LayoutParams(doweDp(28), doweDp(28), 0f));
        ArrayList<DoweSvgPathEntry> paths = new ArrayList<>();
        paths.add(new DoweSvgPathEntry("M0 0h24v24H0z", false, null));
        paths.add(new DoweSvgPathEntry("m4.397 4.554l.073-.084a.75.75 0 0 1 .976-.073l.084.073L12 10.939l6.47-6.47a.75.75 0 1 1 1.06 1.061L13.061 12l6.47 6.47a.75.75 0 0 1 .072.976l-.073.084a.75.75 0 0 1-.976.073l-.084-.073L12 13.061l-6.47 6.47a.75.75 0 0 1-1.06-1.061L10.939 12l-6.47-6.47a.75.75 0 0 1-.072-.976l.073-.084z", true, null));
        DoweSvgView icon = new DoweSvgView(this, 0f, 0f, 24f, 24f, DOWE_MUTED_TEXT, paths);
        icon.setImportantForAccessibility(View.IMPORTANT_FOR_ACCESSIBILITY_NO);
        close.addView(icon, new FrameLayout.LayoutParams(doweDp(18), doweDp(18), Gravity.CENTER));
        LinearLayout.LayoutParams closeParams = (LinearLayout.LayoutParams) close.getLayoutParams();
        closeParams.setMargins(doweDp(8), 0, 0, 0);
        close.setLayoutParams(closeParams);
        doweAdd(panel, close);
        final int gravity = (position.startsWith("top") ? Gravity.TOP : Gravity.BOTTOM)
            | (position.endsWith("left") ? Gravity.START : Gravity.END);
        PopupWindow popup = new PopupWindow(panel, ViewGroup.LayoutParams.WRAP_CONTENT, ViewGroup.LayoutParams.WRAP_CONTENT, false);
        popup.setBackgroundDrawable(new android.graphics.drawable.ColorDrawable(Color.TRANSPARENT));
        close.setOnClickListener(view -> popup.dismiss());
        root.post(() -> {
            if (root.getWindowToken() == null) return;
            int toastWidth = Math.max(doweDp(1), Math.min(doweDp(420), Math.max(0, root.getWidth() - doweDp(32))));
            popup.setWidth(toastWidth);
            ViewGroup.LayoutParams panelParams = panel.getLayoutParams();
            if (panelParams != null) {
                panelParams.width = toastWidth;
                panel.setLayoutParams(panelParams);
            }
            popup.showAtLocation(root, gravity, doweDp(16), doweDp(16));
            root.postDelayed(() -> { if (popup.isShowing()) popup.dismiss(); }, Math.max(500, step.duration == null ? 4000 : step.duration));
        });
    }

    void doweRunStartup(String[] ids) {
