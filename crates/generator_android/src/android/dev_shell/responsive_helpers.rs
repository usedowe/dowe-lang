fn dev_activity_responsive_helpers() -> &'static str {
    r#"    private Integer doweResponsiveInt(int viewportWidth, Integer xs, Integer sm, Integer md, Integer lg, Integer xl) {
        Integer value = null;
        if (viewportWidth >= 0 && xs != null) {
            value = xs;
        }
        if (viewportWidth >= 640 && sm != null) {
            value = sm;
        }
        if (viewportWidth >= 768 && md != null) {
            value = md;
        }
        if (viewportWidth >= 1024 && lg != null) {
            value = lg;
        }
        if (viewportWidth >= 1280 && xl != null) {
            value = xl;
        }
        return value;
    }

    private float[] doweResponsiveTracks(int viewportWidth, float[] xs, float[] sm, float[] md, float[] lg, float[] xl) {
        float[] value = null;
        if (viewportWidth >= 0 && xs != null) value = xs;
        if (viewportWidth >= 640 && sm != null) value = sm;
        if (viewportWidth >= 768 && md != null) value = md;
        if (viewportWidth >= 1024 && lg != null) value = lg;
        if (viewportWidth >= 1280 && xl != null) value = xl;
        return value;
    }

    private Float doweResponsiveFloat(int viewportWidth, Float xs, Float sm, Float md, Float lg, Float xl) {
        Float value = null;
        if (viewportWidth >= 0 && xs != null) {
            value = xs;
        }
        if (viewportWidth >= 640 && sm != null) {
            value = sm;
        }
        if (viewportWidth >= 768 && md != null) {
            value = md;
        }
        if (viewportWidth >= 1024 && lg != null) {
            value = lg;
        }
        if (viewportWidth >= 1280 && xl != null) {
            value = xl;
        }
        return value;
    }

    private String doweResponsiveString(int viewportWidth, String xs, String sm, String md, String lg, String xl) {
        String value = null;
        if (viewportWidth >= 0 && xs != null) {
            value = xs;
        }
        if (viewportWidth >= 640 && sm != null) {
            value = sm;
        }
        if (viewportWidth >= 768 && md != null) {
            value = md;
        }
        if (viewportWidth >= 1024 && lg != null) {
            value = lg;
        }
        if (viewportWidth >= 1280 && xl != null) {
            value = xl;
        }
        return value;
    }

    private Boolean doweResponsiveBool(int viewportWidth, Boolean xs, Boolean sm, Boolean md, Boolean lg, Boolean xl) {
        Boolean value = null;
        if (viewportWidth >= 0 && xs != null) {
            value = xs;
        }
        if (viewportWidth >= 640 && sm != null) {
            value = sm;
        }
        if (viewportWidth >= 768 && md != null) {
            value = md;
        }
        if (viewportWidth >= 1024 && lg != null) {
            value = lg;
        }
        if (viewportWidth >= 1280 && xl != null) {
            value = xl;
        }
        return value;
    }

    private boolean doweShow(Boolean value) {
        return value == null || value;
    }

    private String doweFontName(String value) {
        return value == null ? "__DOWE_DEFAULT_FONT__" : value;
    }

    private int doweDp(int value) {
        return Math.round(value * getResources().getDisplayMetrics().density);
    }

    private float doweDp(float value) {
        return value * getResources().getDisplayMetrics().density;
    }

    private int doweViewportHeight(int inset) {
        int safeInsets = scrollView == null ? 0 : scrollView.getPaddingTop() + scrollView.getPaddingBottom();
        int safeInsetsDp = Math.round(safeInsets / getResources().getDisplayMetrics().density);
        return Math.max(0, getResources().getConfiguration().screenHeightDp - safeInsetsDp - inset);
    }

    private int dowePercentSize(int percentage) {
        return Integer.MIN_VALUE + percentage;
    }

    private boolean doweIsPercentSize(Integer value) {
        return value != null && value >= Integer.MIN_VALUE + 10 && value <= Integer.MIN_VALUE + 100;
    }

    private int dowePercentValue(Integer value) {
        return value - Integer.MIN_VALUE;
    }

    private int doweDimension(Integer value) {
        if (value == null || doweIsPercentSize(value)) {
            return ViewGroup.LayoutParams.WRAP_CONTENT;
        }
        if (value == ViewGroup.LayoutParams.MATCH_PARENT || value == ViewGroup.LayoutParams.WRAP_CONTENT) {
            return value;
        }
        return doweDp(value);
    }

    private void doweApplyPercentWidth(View view, Integer value) {
        doweApplyPercentWidth(view, value, false);
    }

    private void doweApplyPercentMinWidth(View view, Integer value) {
        doweApplyPercentWidth(view, value, true);
    }

    private void doweApplyPercentWidth(View view, Integer value, boolean minimum) {
        if (!doweIsPercentSize(value)) {
            return;
        }
        int percentage = dowePercentValue(value);
        view.post(() -> {
            if (!(view.getParent() instanceof View)) {
                return;
            }
            View parent = (View) view.getParent();
            Runnable apply = () -> {
                int availableWidth = Math.max(0, parent.getWidth() - parent.getPaddingLeft() - parent.getPaddingRight());
                int resolved = Math.round(availableWidth * percentage / 100f);
                if (minimum) {
                    view.setMinimumWidth(resolved);
                    return;
                }
                ViewGroup.LayoutParams params = view.getLayoutParams();
                if (params != null && params.width != resolved) {
                    params.width = resolved;
                    view.setLayoutParams(params);
                }
            };
            parent.addOnLayoutChangeListener((current, left, top, right, bottom, oldLeft, oldTop, oldRight, oldBottom) -> apply.run());
            apply.run();
        });
    }

    private void doweConstrain(View view, Integer maxWidth, Integer maxHeight) {
        view.addOnLayoutChangeListener((current, left, top, right, bottom, oldLeft, oldTop, oldRight, oldBottom) -> {
            ViewGroup.LayoutParams params = current.getLayoutParams();
            if (params == null) {
                return;
            }
            int widthLimit = doweConstraintLimit(current, maxWidth, true);
            int heightLimit = doweConstraintLimit(current, maxHeight, false);
            int width = current.getMeasuredWidth() > widthLimit ? widthLimit : params.width;
            int height = current.getMeasuredHeight() > heightLimit ? heightLimit : params.height;
            if (width != params.width || height != params.height) {
                params.width = width;
                params.height = height;
                current.setLayoutParams(params);
            }
        });
    }

    private int doweConstraintLimit(View view, Integer value, boolean horizontal) {
        if (value == null || value == ViewGroup.LayoutParams.WRAP_CONTENT) {
            return Integer.MAX_VALUE;
        }
        if (value != ViewGroup.LayoutParams.MATCH_PARENT) {
            return doweDp(value);
        }
        if (!(view.getParent() instanceof View)) {
            return Integer.MAX_VALUE;
        }
        View parent = (View) view.getParent();
        int available = horizontal
            ? parent.getWidth() - parent.getPaddingLeft() - parent.getPaddingRight()
            : parent.getHeight() - parent.getPaddingTop() - parent.getPaddingBottom();
        return available > 0 ? available : Integer.MAX_VALUE;
    }

    private int doweColor(Integer value, int fallback) {
        return value == null ? fallback : value;
    }

    private float doweTextSize(Float value, float fallback) {
        return value == null ? fallback : value;
    }

    private float doweFloat(Float value, float fallback) {
        return value == null ? fallback : value;
    }

    private int doweTextWeight(Integer value, int fallback) {
        return value == null ? fallback : value;
    }

    private float doweFluidTextSize(float min, float preferredBase, float preferredViewport, float max) {
        return Math.max(min, Math.min(preferredBase + viewportWidth * preferredViewport / 100f, max));
    }
}
"#
}
