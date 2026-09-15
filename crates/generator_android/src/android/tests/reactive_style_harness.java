import java.util.*;

class ReactiveStyleHarness {
    static final int DOWE_COMPONENT_TAG = 1, DOWE_CARD_ROLE_TAG = 2, DOWE_SCHEME_TAG = 3,
        DOWE_VARIANT_TAG = 4, DOWE_SIZE_TAG = 5, DOWE_SCHEME_FALLBACK_TAG = 6,
        DOWE_VARIANT_FALLBACK_TAG = 7, DOWE_DISABLED_PATH_TAG = 8, DOWE_RADIUS = 8;
    static class View {
        Map<Integer, Object> tags = new HashMap<>();
        View parent;
        Object background = "original";
        int minHeight;
        Object getTag(int key) { return tags.get(key); }
        Object getParent() { return parent; }
        void setBackground(Object value) { background = value; }
        void setMinimumHeight(int value) { minHeight = value; }
        void setPadding(int a, int b, int c, int d) { }
        void setEnabled(boolean value) { }
        void setAlpha(float value) { }
    }
    static class ViewGroup extends View {
        List<View> children = new ArrayList<>();
        void add(View view) { children.add(view); view.parent = this; }
        int getChildCount() { return children.size(); }
        View getChildAt(int index) { return children.get(index); }
    }
    static class TextView extends View {
        int color = 123;
        float textSize = 14;
        void setTextColor(int value) { color = value; }
        void setTextSize(int unit, float value) { textSize = value; }
    }
    Map<String, String> state = new HashMap<>();
    String doweReactiveEnum(Object path, String property, String fallback) {
        return path instanceof String ? state.getOrDefault(path, fallback) : fallback;
    }
    void doweApplyReactiveStyles(View view) { }
    boolean doweBool(String path, Object item) { return false; }
    int doweButtonContent(String variant, String scheme) { return (variant + scheme).hashCode(); }
    int doweButtonContainer(String variant, String scheme) { return ("button" + variant + scheme).hashCode(); }
    int doweCardTitle(String variant, String scheme) { return ("title" + variant + scheme).hashCode(); }
    int doweCardContent(String variant, String scheme) { return ("content" + variant + scheme).hashCode(); }
    int doweCardContainer(String variant, String scheme) { return ("card" + variant + scheme).hashCode(); }
    int doweCardBorder(String variant, String scheme) { return ("border" + variant + scheme).hashCode(); }
    Object doweInputBackground(int color, Integer border, int radius) { return color; }
    float doweNativeTextSize(float size) { return size; }
    int doweButtonMinHeight(String size) { return 40; }
    int doweButtonHorizontalPadding(String size) { return 12; }
    int doweButtonVerticalPadding(String size) { return 8; }
    __DOWE_METHODS__
    static void check(boolean value, String message) {
        if (!value) throw new AssertionError(message);
    }
    public static void main(String[] args) {
        ReactiveStyleHarness runtime = new ReactiveStyleHarness();
        ViewGroup root = new ViewGroup();
        TextView text = new TextView();
        TextView outlinedButton = new TextView();
        TextView sizeOnly = new TextView();
        sizeOnly.tags.put(DOWE_SIZE_TAG, "size");
        runtime.state.put("size", "lg");
        ViewGroup staticCard = new ViewGroup();
        staticCard.tags.put(DOWE_COMPONENT_TAG, "card");
        TextView cardText = new TextView();
        staticCard.add(cardText);
        root.add(text); root.add(outlinedButton); root.add(sizeOnly); root.add(staticCard);
        for (int i = 0; i < 60; i++) runtime.doweRefreshReactiveControls(root);
        check(text.color == 123, "Signal refresh overwrote ordinary text color");
        check(cardText.color == 123, "Signal refresh overwrote static Card text color");
        check(outlinedButton.color == 123, "Signal refresh overwrote static button color");
        check(sizeOnly.color == 123 && sizeOnly.textSize == 18, "Size refresh changed color or lost size");
        check("original".equals(staticCard.background) && "original".equals(outlinedButton.background), "Static background changed");
        TextView reactiveButton = new TextView();
        reactiveButton.tags.put(DOWE_SCHEME_TAG, "scheme");
        reactiveButton.tags.put(DOWE_VARIANT_FALLBACK_TAG, "outlined");
        root.add(reactiveButton);
        ViewGroup reactiveCard = new ViewGroup();
        reactiveCard.tags.put(DOWE_COMPONENT_TAG, "card");
        reactiveCard.tags.put(DOWE_SCHEME_TAG, "scheme");
        TextView title = new TextView(), content = new TextView();
        title.tags.put(DOWE_CARD_ROLE_TAG, "title");
        content.tags.put(DOWE_CARD_ROLE_TAG, "content");
        reactiveCard.add(title); reactiveCard.add(content); root.add(reactiveCard);
        for (String scheme : new String[] {"primary", "success"}) {
            runtime.state.put("scheme", scheme);
            runtime.doweRefreshReactiveControls(root);
            check(reactiveButton.color == runtime.doweButtonContent("outlined", scheme), "Reactive button lost scheme");
            check(title.color == runtime.doweCardTitle("solid", scheme), "Reactive Card lost title color");
            check(content.color == runtime.doweCardContent("solid", scheme), "Reactive Card lost content color");
            check(text.color == 123 && cardText.color == 123, "Reactive neighbor changed static text");
        }
    }
}
