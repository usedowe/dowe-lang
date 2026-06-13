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

    private int doweDimension(Integer value) {
        if (value == null) {
            return ViewGroup.LayoutParams.WRAP_CONTENT;
        }
        if (value == ViewGroup.LayoutParams.MATCH_PARENT) {
            return ViewGroup.LayoutParams.MATCH_PARENT;
        }
        return doweDp(value);
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
