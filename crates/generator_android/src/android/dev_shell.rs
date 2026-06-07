fn replace_android_font_support(
    output: &mut String,
    font_config: &FontConfig,
    font_families: &BTreeSet<FontFamily>,
) {
    let start = output
        .find("private enum class DoweFont {")
        .expect("font support start");
    let end = output
        .find("private sealed class DoweSize {")
        .expect("font support end");
    output.replace_range(
        start..end,
        &android_font_support(font_config, font_families),
    );
}

fn android_font_support(font_config: &FontConfig, font_families: &BTreeSet<FontFamily>) -> String {
    let enum_cases = font_families
        .iter()
        .map(|font| format!("    {}", font_name(*font)))
        .collect::<Vec<_>>()
        .join(",\n");
    let font_objects = font_families
        .iter()
        .filter(|font| font.catalog_entry().package_assets)
        .map(|font| {
            let entry = font.catalog_entry();
            let fonts = entry
                .weights
                .iter()
                .map(|weight| {
                    format!(
                        "        Font(R.font.{}, {})",
                        android_font_resource_name(weight.asset_stem),
                        compose_text_weight(weight.weight)
                    )
                })
                .collect::<Vec<_>>()
                .join(",\n");
            format!("    val {} = FontFamily(\n{fonts}\n    )", font.as_str())
        })
        .collect::<Vec<_>>()
        .join("\n");
    let branches = font_families
        .iter()
        .map(|font| {
            format!(
                "        DoweFont.{} -> {}",
                font_name(*font),
                compose_font_family_ref(*font)
            )
        })
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        r#"private enum class DoweFont {{
{enum_cases}
}}

private object DoweFonts {{
{font_objects}
}}

private fun doweFontFamily(value: DoweFont?): FontFamily {{
    return when (value) {{
{branches}
        null -> {}
    }}
}}

"#,
        compose_font_family_ref(font_config.default_family)
    )
}

fn dev_activity(
    routes: &[ViewRoute],
    font_config: &FontConfig,
    design_config: &DesignConfig,
    environment: &[(String, String)],
    app_bundle: &str,
) -> String {
    let mut output = String::from(
        r#"package dev.dowe.generated;

import android.app.Activity;
import android.content.ClipData;
import android.content.ClipboardManager;
import android.content.Context;
import android.content.Intent;
import android.graphics.Color;
import android.graphics.Insets;
import android.graphics.Canvas;
import android.graphics.Matrix;
import android.graphics.Paint;
import android.graphics.Path;
import android.graphics.Typeface;
import android.graphics.drawable.GradientDrawable;
import android.net.Uri;
import android.os.Build;
import android.os.Bundle;
import android.os.Handler;
import android.os.Looper;
import android.text.SpannableString;
import android.text.style.ForegroundColorSpan;
import android.text.Editable;
import android.text.TextWatcher;
import android.view.Gravity;
import android.view.View;
import android.view.ViewGroup;
import android.view.WindowInsets;
import android.webkit.WebView;
import android.widget.Button;
import android.widget.EditText;
import android.widget.FrameLayout;
import android.widget.HorizontalScrollView;
import android.widget.ImageView;
import android.widget.LinearLayout;
import android.widget.MediaController;
import android.widget.ScrollView;
import android.widget.PopupWindow;
import android.widget.TextView;
import android.widget.VideoView;
import java.io.BufferedReader;
import java.io.InputStream;
import java.io.InputStreamReader;
import java.net.HttpURLConnection;
import java.net.URL;
import java.util.ArrayList;
import java.util.HashMap;
import java.util.HashSet;
import java.util.List;
import java.util.Map;
import java.util.Objects;
import java.util.function.Consumer;
import org.json.JSONArray;
import org.json.JSONObject;

@SuppressWarnings({"unchecked", "deprecation"})
public class DoweDevActivity extends Activity {
    private static final int DOWE_JUSTIFY_START = 0;
    private static final int DOWE_JUSTIFY_CENTER = 1;
    private static final int DOWE_JUSTIFY_END = 2;
    private static final int DOWE_JUSTIFY_BETWEEN = 3;
    private static final int DOWE_JUSTIFY_AROUND = 4;
    private static final int DOWE_JUSTIFY_EVENLY = 5;
    private static final int DOWE_ALIGN_START = 0;
    private static final int DOWE_ALIGN_CENTER = 1;
    private static final int DOWE_ALIGN_END = 2;
    private static final int DOWE_ALIGN_STRETCH = 3;
    private static final int DOWE_ALIGN_BASELINE = 4;
"#,
    );
    if app_bundle != "dev.dowe.generated" {
        output.insert_str(
            "package dev.dowe.generated;\n\n".len(),
            &format!("import {}.R;\n", app_bundle),
        );
    }
    output.push_str(&dev_design_constants(design_config.default_theme()));
    output.push_str(&format!(
        "    private LinearLayout root;\n    private ScrollView scrollView;\n    private int viewportWidth;\n    private String currentPath = \"{}\";\n    private String currentFragment = null;\n    private boolean externalOpen = false;\n    private final ArrayList<DoweRouteEntry> backStack = new ArrayList<>();\n    private final HashMap<String, Object> doweState = new HashMap<>();\n    private final HashMap<String, Object> doweInitial = new HashMap<>();\n    private final HashMap<String, DoweAction> doweActions = new HashMap<>();\n    private final HashMap<String, View> sectionViews = new HashMap<>();\n    private final HashSet<String> doweLoaded = new HashSet<>();\n\n",
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
    output.push_str(
        r#"    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        doweConfigureWindow();
        FrameLayout background = new FrameLayout(this);
        background.setBackgroundColor(DOWE_BACKGROUND);
        background.setLayoutParams(new FrameLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.MATCH_PARENT));
        root = new LinearLayout(this);
        root.setOrientation(LinearLayout.VERTICAL);
        root.setGravity(Gravity.TOP | Gravity.START);
        root.setBackgroundColor(DOWE_BACKGROUND);
        root.setLayoutParams(new LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.MATCH_PARENT));
        scrollView = new ScrollView(this);
        scrollView.setFillViewport(true);
        scrollView.addView(root);
        background.addView(scrollView, new FrameLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.MATCH_PARENT));
        viewportWidth = getResources().getConfiguration().screenWidthDp;
        doweInitializeState();
        doweApplyIntentRoute();
        doweRegisterBackHandler();
        setContentView(background);
        doweApplySystemInsets(scrollView);
        renderCurrentRoute();
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
        getWindow().getDecorView().setSystemUiVisibility(
            View.SYSTEM_UI_FLAG_LAYOUT_STABLE |
            View.SYSTEM_UI_FLAG_LAYOUT_FULLSCREEN |
            View.SYSTEM_UI_FLAG_LAYOUT_HIDE_NAVIGATION |
            View.SYSTEM_UI_FLAG_LIGHT_STATUS_BAR |
            View.SYSTEM_UI_FLAG_LIGHT_NAVIGATION_BAR
        );
    }

    private void doweApplySystemInsets(View view) {
        view.setOnApplyWindowInsetsListener((target, insets) -> {
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
            return insets;
        });
        view.requestApplyInsets();
    }

    @Override
    protected void onNewIntent(Intent intent) {
        super.onNewIntent(intent);
        setIntent(intent);
        doweApplyIntentRoute();
        renderCurrentRoute();
    }

    @Override
    public void onBackPressed() {
        doweBack();
    }

    private void doweRegisterBackHandler() {
        if (Build.VERSION.SDK_INT >= 33) {
            getOnBackInvokedDispatcher().registerOnBackInvokedCallback(
                android.window.OnBackInvokedDispatcher.PRIORITY_DEFAULT,
                this::doweBack
            );
        }
    }

    private void renderCurrentRoute() {
        renderCurrentRoute(true);
    }

    private void renderCurrentRoute(boolean scrollToFragment) {
        root.removeAllViews();
        sectionViews.clear();
        externalOpen = false;
"#,
    );

    for (index, route) in routes.iter().enumerate() {
        let branch = if index == 0 { "if" } else { "else if" };
        output.push_str(&format!(
            "        {branch} (\"{}\".equals(currentPath)) {{\n            {}(root);\n        }}\n",
            escape_java(&route.route_path),
            dev_route_method_name(&route.route_path)
        ));
    }

    if let Some(route) = routes.first() {
        output.push_str(&format!(
            "        else {{\n            currentPath = \"{}\";\n            {}(root);\n        }}\n",
            escape_java(&route.route_path),
            dev_route_method_name(&route.route_path)
        ));
    }

    output.push_str(
        "        doweAutoload();\n        if (scrollToFragment) {\n            doweScrollToFragment();\n        }\n    }\n\n",
    );

    let (layouts, route_layouts) = reusable_dev_layouts(routes);
    for (route, layout_index) in routes.iter().zip(route_layouts) {
        let context = ComposeReactiveContext::default();
        let mut counter = 0;
        if let Some(layout_index) = layout_index {
            let page_method = dev_route_page_method_name(&route.route_path);
            output.push_str(&format!(
                "    private void {}(LinearLayout root) {{\n        renderLayout{layout_index}(root, this::{page_method});\n    }}\n\n",
                dev_route_method_name(&route.route_path),
            ));
            output.push_str(&format!(
                "    private void {page_method}(ViewGroup root) {{\n        int viewportWidth = this.viewportWidth;\n"
            ));
            render_dev_android_node(
                &route.page_tree,
                "root",
                None,
                false,
                &mut counter,
                &mut output,
                None,
                None,
                &context,
                None,
            );
            output.push_str("    }\n\n");
        } else {
            output.push_str(&format!(
                "    private void {}(LinearLayout root) {{\n        int viewportWidth = this.viewportWidth;\n",
                dev_route_method_name(&route.route_path)
            ));
            let tree = compose_tree(&route.layout_tree, &route.page_tree);
            render_dev_android_node(
                &tree,
                "root",
                None,
                false,
                &mut counter,
                &mut output,
                None,
                None,
                &context,
                None,
            );
            output.push_str("    }\n\n");
        }
    }
    for (index, layout) in layouts.into_iter().enumerate() {
        output.push_str(&format!(
            "    private void renderLayout{index}(ViewGroup root, Consumer<ViewGroup> page) {{\n        int viewportWidth = this.viewportWidth;\n"
        ));
        let context = ComposeReactiveContext::default();
        let mut counter = 0;
        render_dev_android_node(
            layout,
            "root",
            None,
            false,
            &mut counter,
            &mut output,
            None,
            None,
            &context,
            Some("page.accept"),
        );
        output.push_str("    }\n\n");
    }

    output.push_str("    private void doweInitializeState() {\n");
    for route in routes {
        let tree = compose_tree(&route.layout_tree, &route.page_tree);
        let reactive = dev_reactive_route(&tree);
        for initial in reactive.initial {
            output.push_str(&format!("        {initial}\n"));
        }
        for action in reactive.actions {
            output.push_str(&format!("        {action}\n"));
        }
    }
    output.push_str("    }\n\n    private void doweAutoload() {\n");
    for route in routes {
        let tree = compose_tree(&route.layout_tree, &route.page_tree);
        let reactive = dev_reactive_route(&tree);
        for id in reactive.autoload {
            output.push_str(&format!(
                "        if (\"{}\".equals(currentPath) && doweLoaded.add(\"{}\")) {{\n            doweRunAction(\"{}\", null);\n        }}\n",
                escape_java(&route.route_path),
                escape_java(&id),
                escape_java(&id)
            ));
        }
    }
    output.push_str("    }\n\n");

    output.push_str(&format!(
        r#"    private void doweNavigate(String operation, String target, String fragment) {{
        String path = target.isEmpty() ? currentPath : target;
        if (!doweCanRoute(path)) {{
            return;
        }}
        String resolvedFragment = doweCanSection(path, fragment) ? fragment : null;
        if (path.equals(currentPath) && Objects.equals(resolvedFragment, currentFragment)) {{
            return;
        }}
        if ("replace".equals(operation)) {{
            currentPath = path;
            currentFragment = resolvedFragment;
        }} else {{
            backStack.add(new DoweRouteEntry(currentPath, currentFragment));
            currentPath = path;
            currentFragment = resolvedFragment;
        }}
        renderCurrentRoute();
    }}

    private void doweBack() {{
        if (externalOpen) {{
            renderCurrentRoute();
        }} else if (!backStack.isEmpty()) {{
            DoweRouteEntry previous = backStack.remove(backStack.size() - 1);
            currentPath = previous.path;
            currentFragment = previous.fragment;
            renderCurrentRoute();
        }} else if (!currentPath.equals("{}") || currentFragment != null) {{
            currentPath = "{}";
            currentFragment = null;
            renderCurrentRoute();
        }}
    }}

    private void doweOpenExternal(String mode, String target) {{
        if ("webview".equals(mode)) {{
            root.removeAllViews();
            externalOpen = true;
            WebView webView = new WebView(this);
            webView.setLayoutParams(new LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.MATCH_PARENT));
            webView.loadUrl(target);
            root.addView(webView);
        }} else {{
            startActivity(new Intent(Intent.ACTION_VIEW, Uri.parse(target)));
        }}
    }}

    private void doweApplyIntentRoute() {{
        Uri data = getIntent() == null ? null : getIntent().getData();
        String path = data == null ? null : data.getPath();
        if (path == null || path.isEmpty()) {{
            path = "/";
        }}
        if (doweCanRoute(path)) {{
            currentPath = path;
            String fragment = data == null ? null : data.getFragment();
            currentFragment = doweCanSection(path, fragment) ? fragment : null;
            backStack.clear();
        }}
    }}

    private boolean doweCanRoute(String path) {{
"#,
        escape_java(routes_first_path(routes)),
        escape_java(routes_first_path(routes))
    ));

    if routes.is_empty() {
        output.push_str("        return false;\n");
    } else {
        let route_checks = routes
            .iter()
            .map(|route| format!("\"{}\".equals(path)", escape_java(&route.route_path)))
            .collect::<Vec<_>>()
            .join(" || ");
        output.push_str(&format!("        return {route_checks};\n"));
    }

    output.push_str(
        "    }\n\n    private boolean doweCanSection(String path, String fragment) {\n        if (fragment == null) {\n            return true;\n        }\n",
    );
    for route in routes {
        let section_checks = route
            .sections
            .iter()
            .map(|section| format!("\"{}\".equals(fragment)", escape_java(&section.id)))
            .collect::<Vec<_>>()
            .join(" || ");
        output.push_str(&format!(
            "        if (\"{}\".equals(path)) {{\n            return {};\n        }}\n",
            escape_java(&route.route_path),
            if section_checks.is_empty() {
                "false".to_string()
            } else {
                section_checks
            }
        ));
    }
    output.push_str("        return false;\n    }\n\n");

    output.push_str(
        r#"    private LinearLayout doweContainer(boolean horizontal) {
        LinearLayout view = new LinearLayout(this);
        view.setOrientation(horizontal ? LinearLayout.HORIZONTAL : LinearLayout.VERTICAL);
        view.setLayoutParams(new LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT));
        return view;
    }

    private DoweFlexLayout doweFlex(Integer justify, Integer align, Integer gap) {
        DoweFlexLayout view = new DoweFlexLayout(
            this,
            justify == null ? DOWE_JUSTIFY_START : justify,
            align == null ? DOWE_ALIGN_STRETCH : align,
            gap == null ? 0 : doweDp(gap)
        );
        view.setLayoutParams(new LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT));
        return view;
    }

    private LinearLayout doweCard(int backgroundColor, Integer borderColor) {
        LinearLayout view = doweContainer(false);
        view.setBackground(borderColor == null
            ? doweBackground(backgroundColor, DOWE_RADIUS_BOX)
            : doweInputBackground(backgroundColor, borderColor, DOWE_RADIUS_BOX));
        return view;
    }

    private LinearLayout doweTable(String dataPath, String[] fields, String[] labels, int[] alignments, String[] widths, int tableSize, boolean striped, boolean bordered, boolean dividers, String emptyTitle, String emptyDescription, int backgroundColor, int contentColor, Integer borderColor) {
        LinearLayout view = doweContainer(false);
        view.setBackground(borderColor == null
            ? doweBackground(backgroundColor, DOWE_RADIUS_BOX)
            : doweInputBackground(backgroundColor, borderColor, DOWE_RADIUS_BOX));
        HorizontalScrollView scroll = new HorizontalScrollView(this);
        scroll.setLayoutParams(new LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT));
        LinearLayout table = doweContainer(false);
        table.setLayoutParams(new HorizontalScrollView.LayoutParams(ViewGroup.LayoutParams.WRAP_CONTENT, ViewGroup.LayoutParams.WRAP_CONTENT));
        table.setMinimumWidth(doweTableMinimumWidth(widths, tableSize));
        LinearLayout header = doweTableRow();
        header.setBackgroundColor(doweAlpha(contentColor, 0.08f));
        for (int index = 0; index < labels.length; index += 1) {
            TextView cell = doweTableCell(labels[index], contentColor, tableSize, true, alignments[index], widths[index]);
            doweAdd(header, cell);
        }
        doweAdd(table, header);
        ArrayList<Map<String, Object>> rows = doweRows(dataPath);
        if (rows.isEmpty()) {
            LinearLayout empty = doweContainer(false);
            empty.setGravity(Gravity.CENTER);
            empty.setMinimumHeight(doweDp(120));
            empty.setPadding(doweDp(16), doweDp(16), doweDp(16), doweDp(16));
            TextView title = doweText(emptyTitle, contentColor, tableSize == 2 ? 20f : tableSize == 0 ? 16f : 18f, 700, 0f, 1.2f, "sans");
            title.setGravity(Gravity.CENTER);
            TextView description = doweText(emptyDescription, doweAlpha(contentColor, 0.68f), tableSize == 2 ? 15f : tableSize == 0 ? 13f : 14f, 400, 0f, 1.25f, "sans");
            description.setGravity(Gravity.CENTER);
            doweAdd(empty, title);
            doweAdd(empty, description, 4, false);
            doweAdd(table, empty);
        } else {
            for (int rowIndex = 0; rowIndex < rows.size(); rowIndex += 1) {
                LinearLayout row = doweTableRow();
                if (striped && rowIndex % 2 == 1) {
                    row.setBackgroundColor(doweAlpha(contentColor, 0.05f));
                }
                for (int columnIndex = 0; columnIndex < fields.length; columnIndex += 1) {
                    TextView cell = doweTableCell(doweTableValue(rows.get(rowIndex), fields[columnIndex]), contentColor, tableSize, false, alignments[columnIndex], widths[columnIndex]);
                    if (bordered && columnIndex < fields.length - 1) {
                        cell.setBackground(doweInputBackground(Color.TRANSPARENT, doweAlpha(contentColor, 0.12f), 0));
                    }
                    doweAdd(row, cell);
                }
                doweAdd(table, row);
                if (dividers && rowIndex < rows.size() - 1) {
                    View divider = new View(this);
                    divider.setBackgroundColor(doweAlpha(contentColor, 0.12f));
                    divider.setLayoutParams(new LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, doweDp(1)));
                    doweAdd(table, divider);
                }
            }
        }
        scroll.addView(table);
        doweAdd(view, scroll);
        return view;
    }

    private LinearLayout doweTableRow() {
        LinearLayout row = new LinearLayout(this);
        row.setOrientation(LinearLayout.HORIZONTAL);
        row.setLayoutParams(new LinearLayout.LayoutParams(ViewGroup.LayoutParams.WRAP_CONTENT, ViewGroup.LayoutParams.WRAP_CONTENT));
        return row;
    }

    private TextView doweTableCell(String value, int color, int tableSize, boolean header, int gravity, String width) {
        float textSize = tableSize == 2 ? 16f : tableSize == 0 ? 12f : 14f;
        TextView cell = doweText(value, color, textSize, header ? 700 : 400, 0f, 1.25f, "sans");
        int horizontal = tableSize == 2 ? 20 : tableSize == 0 ? 12 : 16;
        int vertical = tableSize == 2 ? (header ? 16 : 20) : tableSize == 0 ? 8 : (header ? 12 : 16);
        cell.setGravity(gravity | Gravity.CENTER_VERTICAL);
        cell.setSingleLine(true);
        cell.setPadding(doweDp(horizontal), doweDp(vertical), doweDp(horizontal), doweDp(vertical));
        cell.setLayoutParams(new LinearLayout.LayoutParams(doweTableColumnWidth(width), ViewGroup.LayoutParams.WRAP_CONTENT));
        return cell;
    }

    private int doweTableColumnWidth(String width) {
        if (width == null || width.isEmpty() || "auto".equals(width) || "min-content".equals(width) || "max-content".equals(width)) {
            return doweDp(160);
        }
        try {
            if (width.endsWith("px")) {
                return doweDp(Math.round(Float.parseFloat(width.substring(0, width.length() - 2))));
            }
            if (width.endsWith("rem")) {
                return doweDp(Math.round(Float.parseFloat(width.substring(0, width.length() - 3)) * 16f));
            }
        } catch (NumberFormatException error) {
        }
        return doweDp(160);
    }

    private int doweTableMinimumWidth(String[] widths, int tableSize) {
        int horizontal = tableSize == 2 ? 20 : tableSize == 0 ? 12 : 16;
        int value = 0;
        for (String width : widths) {
            value += doweTableColumnWidth(width) + doweDp(horizontal * 2);
        }
        return value;
    }

    private String doweTableValue(Map<String, Object> row, String field) {
        String[] parts = field.split("\\.");
        Object current = row.get(parts[0]);
        for (int index = 1; index < parts.length; index += 1) {
            if (!(current instanceof Map)) {
                return "";
            }
            current = ((Map<?, ?>) current).get(parts[index]);
        }
        return current == null ? "" : String.valueOf(current);
    }

    private void doweRegisterSection(String id, View view) {
        if (id != null) {
            sectionViews.put(id, view);
        }
    }

    private void doweScrollToFragment() {
        if (currentFragment == null || scrollView == null) {
            return;
        }
        root.post(() -> {
            View target = sectionViews.get(currentFragment);
            if (target != null) {
                scrollView.scrollTo(0, doweTopRelativeToRoot(target));
            }
        });
    }

    private int doweTopRelativeToRoot(View view) {
        int top = 0;
        View current = view;
        while (current != null && current != root) {
            top += current.getTop();
            Object parent = current.getParent();
            current = parent instanceof View ? (View) parent : null;
        }
        return top;
    }

    private void doweAnimate(View view, String preset) {
        if (preset == null || "none".equals(preset)) {
            return;
        }
        view.setAlpha(0f);
        if ("slideUp".equals(preset)) {
            view.setTranslationY(doweDp(16));
        } else if ("slideDown".equals(preset)) {
            view.setTranslationY(-doweDp(16));
        } else if ("slideLeft".equals(preset)) {
            view.setTranslationX(doweDp(16));
        } else if ("slideRight".equals(preset)) {
            view.setTranslationX(-doweDp(16));
        } else if ("scaleIn".equals(preset)) {
            view.setScaleX(0.96f);
            view.setScaleY(0.96f);
        }
        view.animate().alpha(1f).translationX(0f).translationY(0f).scaleX(1f).scaleY(1f).setDuration(220).start();
    }

    private void doweToggleSideNavSubmenu(View view) {
        view.animate().withEndAction(null).cancel();
        if (view.getVisibility() == View.VISIBLE) {
            view.animate().alpha(0f).translationY(-doweDp(4)).setDuration(140).withEndAction(() -> {
                view.setVisibility(View.GONE);
                view.setAlpha(1f);
                view.setTranslationY(0f);
            }).start();
            return;
        }
        view.setAlpha(0f);
        view.setTranslationY(-doweDp(4));
        view.setVisibility(View.VISIBLE);
        view.animate().alpha(1f).translationY(0f).setDuration(160).withEndAction(null).start();
    }

    private static final class DoweFlexLayout extends ViewGroup {
        private final int justify;
        private final int align;
        private final int gap;

        DoweFlexLayout(Context context, int justify, int align, int gap) {
            super(context);
            this.justify = justify;
            this.align = align;
            this.gap = Math.max(gap, 0);
        }

        @Override
        protected void onMeasure(int widthSpec, int heightSpec) {
            int count = getChildCount();
            int horizontalPadding = getPaddingLeft() + getPaddingRight();
            int verticalPadding = getPaddingTop() + getPaddingBottom();
            int gapTotal = Math.max(0, count - 1) * gap;
            int availableWidth = Math.max(0, MeasureSpec.getSize(widthSpec) - horizontalPadding - gapTotal);
            int fixedWidth = 0;
            int maxHeight = 0;
            float totalWeight = 0f;
            for (int i = 0; i < count; i++) {
                View child = getChildAt(i);
                if (child.getVisibility() == GONE) {
                    continue;
                }
                float weight = doweChildWeight(child);
                if (weight > 0f) {
                    totalWeight += weight;
                } else {
                    ViewGroup.LayoutParams params = doweChildParams(child);
                    int childWidthSpec = getChildMeasureSpec(widthSpec, horizontalPadding + gapTotal, params.width);
                    int childHeightSpec = getChildMeasureSpec(heightSpec, verticalPadding, params.height);
                    child.measure(childWidthSpec, childHeightSpec);
                    fixedWidth += child.getMeasuredWidth();
                    maxHeight = Math.max(maxHeight, child.getMeasuredHeight());
                }
            }
            int remainingWidth = Math.max(0, availableWidth - fixedWidth);
            for (int i = 0; i < count; i++) {
                View child = getChildAt(i);
                if (child.getVisibility() == GONE) {
                    continue;
                }
                float weight = doweChildWeight(child);
                if (weight > 0f) {
                    ViewGroup.LayoutParams params = doweChildParams(child);
                    int weightedWidth = totalWeight > 0f ? Math.round(remainingWidth * (weight / totalWeight)) : 0;
                    int childWidthSpec = MeasureSpec.makeMeasureSpec(weightedWidth, MeasureSpec.EXACTLY);
                    int childHeightSpec = getChildMeasureSpec(heightSpec, verticalPadding, params.height);
                    child.measure(childWidthSpec, childHeightSpec);
                    fixedWidth += child.getMeasuredWidth();
                    maxHeight = Math.max(maxHeight, child.getMeasuredHeight());
                }
            }
            int desiredWidth = horizontalPadding + fixedWidth + gapTotal;
            int desiredHeight = verticalPadding + maxHeight;
            setMeasuredDimension(resolveSize(desiredWidth, widthSpec), resolveSize(desiredHeight, heightSpec));
        }

        @Override
        protected void onLayout(boolean changed, int left, int top, int right, int bottom) {
            int count = getChildCount();
            int visibleCount = 0;
            int childrenWidth = 0;
            for (int i = 0; i < count; i++) {
                View child = getChildAt(i);
                if (child.getVisibility() != GONE) {
                    visibleCount++;
                    childrenWidth += child.getMeasuredWidth();
                }
            }
            int contentWidth = Math.max(0, right - left - getPaddingLeft() - getPaddingRight());
            int contentHeight = Math.max(0, bottom - top - getPaddingTop() - getPaddingBottom());
            int baseGap = Math.max(0, visibleCount - 1) * gap;
            int free = Math.max(0, contentWidth - childrenWidth - baseGap);
            float cursor = getPaddingLeft() + doweLeadingSpace(free, visibleCount);
            float spacing = gap + doweDistributedSpace(free, visibleCount);
            for (int i = 0; i < count; i++) {
                View child = getChildAt(i);
                if (child.getVisibility() == GONE) {
                    continue;
                }
                int childWidth = child.getMeasuredWidth();
                int childHeight = child.getMeasuredHeight();
                int childTop = getPaddingTop() + doweCrossOffset(contentHeight, childHeight);
                int childLeft = Math.round(cursor);
                child.layout(childLeft, childTop, childLeft + childWidth, childTop + childHeight);
                cursor += childWidth + spacing;
            }
        }

        private float doweLeadingSpace(int free, int visibleCount) {
            if (visibleCount <= 0) {
                return 0f;
            }
            if (justify == DOWE_JUSTIFY_CENTER) {
                return free / 2f;
            }
            if (justify == DOWE_JUSTIFY_END) {
                return free;
            }
            if (justify == DOWE_JUSTIFY_AROUND) {
                return free / (visibleCount * 2f);
            }
            if (justify == DOWE_JUSTIFY_EVENLY) {
                return free / (visibleCount + 1f);
            }
            return 0f;
        }

        private float doweDistributedSpace(int free, int visibleCount) {
            if (visibleCount <= 1) {
                return 0f;
            }
            if (justify == DOWE_JUSTIFY_BETWEEN) {
                return free / (visibleCount - 1f);
            }
            if (justify == DOWE_JUSTIFY_AROUND) {
                return free / (float) visibleCount;
            }
            if (justify == DOWE_JUSTIFY_EVENLY) {
                return free / (visibleCount + 1f);
            }
            return 0f;
        }

        private int doweCrossOffset(int contentHeight, int childHeight) {
            if (align == DOWE_ALIGN_CENTER) {
                return Math.max(0, (contentHeight - childHeight) / 2);
            }
            if (align == DOWE_ALIGN_END) {
                return Math.max(0, contentHeight - childHeight);
            }
            return 0;
        }

        private float doweChildWeight(View child) {
            ViewGroup.LayoutParams params = child.getLayoutParams();
            if (params instanceof LinearLayout.LayoutParams) {
                return ((LinearLayout.LayoutParams) params).weight;
            }
            return 0f;
        }

        private ViewGroup.LayoutParams doweChildParams(View child) {
            ViewGroup.LayoutParams params = child.getLayoutParams();
            return params == null ? new ViewGroup.LayoutParams(ViewGroup.LayoutParams.WRAP_CONTENT, ViewGroup.LayoutParams.WRAP_CONTENT) : params;
        }
    }

    private static final class DoweGridLayout extends ViewGroup {
        private final int columns;
        private final int rowGap;
        private final int columnGap;

        DoweGridLayout(Context context, int columns, int rowGap, int columnGap) {
            super(context);
            this.columns = Math.max(columns, 1);
            this.rowGap = rowGap;
            this.columnGap = columnGap;
        }

        @Override
        protected void onMeasure(int widthSpec, int heightSpec) {
            int available = Math.max(0, MeasureSpec.getSize(widthSpec) - getPaddingLeft() - getPaddingRight());
            int cellWidth = Math.max(0, (available - columnGap * (columns - 1)) / columns);
            int totalHeight = getPaddingTop() + getPaddingBottom();
            int rowHeight = 0;
            for (int index = 0; index < getChildCount(); index++) {
                View child = getChildAt(index);
                child.measure(MeasureSpec.makeMeasureSpec(cellWidth, MeasureSpec.EXACTLY), MeasureSpec.makeMeasureSpec(0, MeasureSpec.UNSPECIFIED));
                rowHeight = Math.max(rowHeight, child.getMeasuredHeight());
                if ((index + 1) % columns == 0 || index + 1 == getChildCount()) {
                    totalHeight += rowHeight;
                    if (index + 1 < getChildCount()) {
                        totalHeight += rowGap;
                    }
                    rowHeight = 0;
                }
            }
            setMeasuredDimension(resolveSize(MeasureSpec.getSize(widthSpec), widthSpec), resolveSize(totalHeight, heightSpec));
        }

        @Override
        protected void onLayout(boolean changed, int left, int top, int right, int bottom) {
            int available = Math.max(0, right - left - getPaddingLeft() - getPaddingRight());
            int cellWidth = Math.max(0, (available - columnGap * (columns - 1)) / columns);
            int rowTop = getPaddingTop();
            int rowHeight = 0;
            for (int index = 0; index < getChildCount(); index++) {
                View child = getChildAt(index);
                int column = index % columns;
                int childLeft = getPaddingLeft() + column * (cellWidth + columnGap);
                child.layout(childLeft, rowTop, childLeft + cellWidth, rowTop + child.getMeasuredHeight());
                rowHeight = Math.max(rowHeight, child.getMeasuredHeight());
                if ((index + 1) % columns == 0 || index + 1 == getChildCount()) {
                    rowTop += rowHeight + rowGap;
                    rowHeight = 0;
                }
            }
        }
    }

    private static final class DoweSvgPathEntry {
        private final String data;
        private final boolean currentColor;
        private final Integer color;

        DoweSvgPathEntry(String data, boolean currentColor, Integer color) {
            this.data = data;
            this.currentColor = currentColor;
            this.color = color;
        }
    }

    private static final class DoweSvgPathParser {
        private final String source;
        private int index = 0;
        private char command = 0;

        private DoweSvgPathParser(String source) {
            this.source = source;
        }

        private static Path parse(String source) {
            return new DoweSvgPathParser(source).readPath();
        }

        private Path readPath() {
            Path path = new Path();
            float currentX = 0f;
            float currentY = 0f;
            float startX = 0f;
            float startY = 0f;
            float lastCubicX = 0f;
            float lastCubicY = 0f;
            float lastQuadX = 0f;
            float lastQuadY = 0f;
            boolean hasLastCubic = false;
            boolean hasLastQuad = false;

            while (true) {
                skipSeparators();
                if (index >= source.length()) {
                    return path;
                }
                char next = source.charAt(index);
                if (isCommand(next)) {
                    command = next;
                    index++;
                } else if (command == 0) {
                    return path;
                }
                boolean relative = Character.isLowerCase(command);
                switch (Character.toUpperCase(command)) {
                    case 'M': {
                        Float x = readNumber();
                        Float y = readNumber();
                        if (x == null || y == null) {
                            return path;
                        }
                        currentX = coordinate(x, currentX, relative);
                        currentY = coordinate(y, currentY, relative);
                        path.moveTo(currentX, currentY);
                        startX = currentX;
                        startY = currentY;
                        command = relative ? 'l' : 'L';
                        while (hasNumber()) {
                            x = readNumber();
                            y = readNumber();
                            if (x == null || y == null) {
                                return path;
                            }
                            currentX = coordinate(x, currentX, relative);
                            currentY = coordinate(y, currentY, relative);
                            path.lineTo(currentX, currentY);
                        }
                        hasLastCubic = false;
                        hasLastQuad = false;
                        break;
                    }
                    case 'L': {
                        Float x = readNumber();
                        Float y = readNumber();
                        if (x == null || y == null) {
                            return path;
                        }
                        while (true) {
                            currentX = coordinate(x, currentX, relative);
                            currentY = coordinate(y, currentY, relative);
                            path.lineTo(currentX, currentY);
                            if (!hasNumber()) {
                                break;
                            }
                            x = readNumber();
                            y = readNumber();
                            if (x == null || y == null) {
                                return path;
                            }
                        }
                        hasLastCubic = false;
                        hasLastQuad = false;
                        break;
                    }
                    case 'H': {
                        Float x = readNumber();
                        if (x == null) {
                            return path;
                        }
                        while (true) {
                            currentX = coordinate(x, currentX, relative);
                            path.lineTo(currentX, currentY);
                            if (!hasNumber()) {
                                break;
                            }
                            x = readNumber();
                            if (x == null) {
                                return path;
                            }
                        }
                        hasLastCubic = false;
                        hasLastQuad = false;
                        break;
                    }
                    case 'V': {
                        Float y = readNumber();
                        if (y == null) {
                            return path;
                        }
                        while (true) {
                            currentY = coordinate(y, currentY, relative);
                            path.lineTo(currentX, currentY);
                            if (!hasNumber()) {
                                break;
                            }
                            y = readNumber();
                            if (y == null) {
                                return path;
                            }
                        }
                        hasLastCubic = false;
                        hasLastQuad = false;
                        break;
                    }
                    case 'C': {
                        Float x1 = readNumber();
                        Float y1 = readNumber();
                        Float x2 = readNumber();
                        Float y2 = readNumber();
                        Float x = readNumber();
                        Float y = readNumber();
                        if (x1 == null || y1 == null || x2 == null || y2 == null || x == null || y == null) {
                            return path;
                        }
                        while (true) {
                            float control1X = coordinate(x1, currentX, relative);
                            float control1Y = coordinate(y1, currentY, relative);
                            float control2X = coordinate(x2, currentX, relative);
                            float control2Y = coordinate(y2, currentY, relative);
                            float endX = coordinate(x, currentX, relative);
                            float endY = coordinate(y, currentY, relative);
                            path.cubicTo(control1X, control1Y, control2X, control2Y, endX, endY);
                            currentX = endX;
                            currentY = endY;
                            lastCubicX = control2X;
                            lastCubicY = control2Y;
                            hasLastCubic = true;
                            hasLastQuad = false;
                            if (!hasNumber()) {
                                break;
                            }
                            x1 = readNumber();
                            y1 = readNumber();
                            x2 = readNumber();
                            y2 = readNumber();
                            x = readNumber();
                            y = readNumber();
                            if (x1 == null || y1 == null || x2 == null || y2 == null || x == null || y == null) {
                                return path;
                            }
                        }
                        break;
                    }
                    case 'S': {
                        Float x2 = readNumber();
                        Float y2 = readNumber();
                        Float x = readNumber();
                        Float y = readNumber();
                        if (x2 == null || y2 == null || x == null || y == null) {
                            return path;
                        }
                        while (true) {
                            float control1X = hasLastCubic ? 2f * currentX - lastCubicX : currentX;
                            float control1Y = hasLastCubic ? 2f * currentY - lastCubicY : currentY;
                            float control2X = coordinate(x2, currentX, relative);
                            float control2Y = coordinate(y2, currentY, relative);
                            float endX = coordinate(x, currentX, relative);
                            float endY = coordinate(y, currentY, relative);
                            path.cubicTo(control1X, control1Y, control2X, control2Y, endX, endY);
                            currentX = endX;
                            currentY = endY;
                            lastCubicX = control2X;
                            lastCubicY = control2Y;
                            hasLastCubic = true;
                            hasLastQuad = false;
                            if (!hasNumber()) {
                                break;
                            }
                            x2 = readNumber();
                            y2 = readNumber();
                            x = readNumber();
                            y = readNumber();
                            if (x2 == null || y2 == null || x == null || y == null) {
                                return path;
                            }
                        }
                        break;
                    }
                    case 'Q': {
                        Float x1 = readNumber();
                        Float y1 = readNumber();
                        Float x = readNumber();
                        Float y = readNumber();
                        if (x1 == null || y1 == null || x == null || y == null) {
                            return path;
                        }
                        while (true) {
                            float controlX = coordinate(x1, currentX, relative);
                            float controlY = coordinate(y1, currentY, relative);
                            float endX = coordinate(x, currentX, relative);
                            float endY = coordinate(y, currentY, relative);
                            path.quadTo(controlX, controlY, endX, endY);
                            currentX = endX;
                            currentY = endY;
                            lastQuadX = controlX;
                            lastQuadY = controlY;
                            hasLastQuad = true;
                            hasLastCubic = false;
                            if (!hasNumber()) {
                                break;
                            }
                            x1 = readNumber();
                            y1 = readNumber();
                            x = readNumber();
                            y = readNumber();
                            if (x1 == null || y1 == null || x == null || y == null) {
                                return path;
                            }
                        }
                        break;
                    }
                    case 'T': {
                        Float x = readNumber();
                        Float y = readNumber();
                        if (x == null || y == null) {
                            return path;
                        }
                        while (true) {
                            float controlX = hasLastQuad ? 2f * currentX - lastQuadX : currentX;
                            float controlY = hasLastQuad ? 2f * currentY - lastQuadY : currentY;
                            float endX = coordinate(x, currentX, relative);
                            float endY = coordinate(y, currentY, relative);
                            path.quadTo(controlX, controlY, endX, endY);
                            currentX = endX;
                            currentY = endY;
                            lastQuadX = controlX;
                            lastQuadY = controlY;
                            hasLastQuad = true;
                            hasLastCubic = false;
                            if (!hasNumber()) {
                                break;
                            }
                            x = readNumber();
                            y = readNumber();
                            if (x == null || y == null) {
                                return path;
                            }
                        }
                        break;
                    }
                    case 'A': {
                        Float rx = readNumber();
                        Float ry = readNumber();
                        Float angle = readNumber();
                        Float largeArc = readNumber();
                        Float sweep = readNumber();
                        Float x = readNumber();
                        Float y = readNumber();
                        if (rx == null || ry == null || angle == null || largeArc == null || sweep == null || x == null || y == null) {
                            return path;
                        }
                        while (true) {
                            float endX = coordinate(x, currentX, relative);
                            float endY = coordinate(y, currentY, relative);
                            addArc(path, currentX, currentY, rx, ry, angle, largeArc != 0f, sweep != 0f, endX, endY);
                            currentX = endX;
                            currentY = endY;
                            hasLastCubic = false;
                            hasLastQuad = false;
                            if (!hasNumber()) {
                                break;
                            }
                            rx = readNumber();
                            ry = readNumber();
                            angle = readNumber();
                            largeArc = readNumber();
                            sweep = readNumber();
                            x = readNumber();
                            y = readNumber();
                            if (rx == null || ry == null || angle == null || largeArc == null || sweep == null || x == null || y == null) {
                                return path;
                            }
                        }
                        break;
                    }
                    case 'Z':
                        path.close();
                        currentX = startX;
                        currentY = startY;
                        hasLastCubic = false;
                        hasLastQuad = false;
                        command = 0;
                        break;
                    default:
                        return path;
                }
            }
        }

        private boolean hasNumber() {
            skipSeparators();
            if (index >= source.length()) {
                return false;
            }
            char value = source.charAt(index);
            return Character.isDigit(value) || value == '+' || value == '-' || value == '.';
        }

        private Float readNumber() {
            skipSeparators();
            int start = index;
            if (index < source.length() && (source.charAt(index) == '+' || source.charAt(index) == '-')) {
                index++;
            }
            boolean digits = false;
            while (index < source.length() && Character.isDigit(source.charAt(index))) {
                index++;
                digits = true;
            }
            if (index < source.length() && source.charAt(index) == '.') {
                index++;
                while (index < source.length() && Character.isDigit(source.charAt(index))) {
                    index++;
                    digits = true;
                }
            }
            if (!digits) {
                index = start;
                return null;
            }
            if (index < source.length() && (source.charAt(index) == 'e' || source.charAt(index) == 'E')) {
                int exponent = index;
                index++;
                if (index < source.length() && (source.charAt(index) == '+' || source.charAt(index) == '-')) {
                    index++;
                }
                int exponentDigits = index;
                while (index < source.length() && Character.isDigit(source.charAt(index))) {
                    index++;
                }
                if (index == exponentDigits) {
                    index = exponent;
                }
            }
            try {
                return Float.parseFloat(source.substring(start, index));
            } catch (NumberFormatException error) {
                return null;
            }
        }

        private void skipSeparators() {
            while (index < source.length()) {
                char value = source.charAt(index);
                if (Character.isWhitespace(value) || value == ',') {
                    index++;
                } else {
                    return;
                }
            }
        }

        private static boolean isCommand(char value) {
            return "MmZzLlHhVvCcSsQqTtAa".indexOf(value) >= 0;
        }

        private static float coordinate(float value, float current, boolean relative) {
            return relative ? current + value : value;
        }

        private static void addArc(Path path, float currentX, float currentY, float rawRx, float rawRy, float angle, boolean largeArc, boolean sweep, float endX, float endY) {
            double rx = Math.abs(rawRx);
            double ry = Math.abs(rawRy);
            if (rx == 0 || ry == 0 || (currentX == endX && currentY == endY)) {
                path.lineTo(endX, endY);
                return;
            }
            double phi = Math.toRadians(angle);
            double cosPhi = Math.cos(phi);
            double sinPhi = Math.sin(phi);
            double dx = (currentX - endX) / 2.0;
            double dy = (currentY - endY) / 2.0;
            double x1p = cosPhi * dx + sinPhi * dy;
            double y1p = -sinPhi * dx + cosPhi * dy;
            double lambda = x1p * x1p / (rx * rx) + y1p * y1p / (ry * ry);
            if (lambda > 1) {
                double factor = Math.sqrt(lambda);
                rx *= factor;
                ry *= factor;
            }
            double rx2 = rx * rx;
            double ry2 = ry * ry;
            double denominator = rx2 * y1p * y1p + ry2 * x1p * x1p;
            if (denominator == 0) {
                path.lineTo(endX, endY);
                return;
            }
            double sign = largeArc == sweep ? -1 : 1;
            double factor = sign * Math.sqrt(Math.max(0, (rx2 * ry2 - rx2 * y1p * y1p - ry2 * x1p * x1p) / denominator));
            double cxp = factor * rx * y1p / ry;
            double cyp = factor * -ry * x1p / rx;
            double cx = cosPhi * cxp - sinPhi * cyp + (currentX + endX) / 2.0;
            double cy = sinPhi * cxp + cosPhi * cyp + (currentY + endY) / 2.0;
            double theta = vectorAngle(1, 0, (x1p - cxp) / rx, (y1p - cyp) / ry);
            double delta = vectorAngle((x1p - cxp) / rx, (y1p - cyp) / ry, (-x1p - cxp) / rx, (-y1p - cyp) / ry);
            if (!sweep && delta > 0) {
                delta -= 2 * Math.PI;
            } else if (sweep && delta < 0) {
                delta += 2 * Math.PI;
            }
            int segments = Math.max(1, (int) Math.ceil(Math.abs(delta) / (Math.PI / 2)));
            double step = delta / segments;
            for (int segment = 0; segment < segments; segment++) {
                double next = theta + step;
                addArcSegment(path, cx, cy, rx, ry, phi, theta, next);
                theta = next;
            }
        }

        private static void addArcSegment(Path path, double cx, double cy, double rx, double ry, double phi, double start, double end) {
            double alpha = 4.0 / 3.0 * Math.tan((end - start) / 4.0);
            double cosStart = Math.cos(start);
            double sinStart = Math.sin(start);
            double cosEnd = Math.cos(end);
            double sinEnd = Math.sin(end);
            float[] control1 = arcPoint(cx, cy, rx, ry, phi, cosStart - alpha * sinStart, sinStart + alpha * cosStart);
            float[] control2 = arcPoint(cx, cy, rx, ry, phi, cosEnd + alpha * sinEnd, sinEnd - alpha * cosEnd);
            float[] point = arcPoint(cx, cy, rx, ry, phi, cosEnd, sinEnd);
            path.cubicTo(control1[0], control1[1], control2[0], control2[1], point[0], point[1]);
        }

        private static float[] arcPoint(double cx, double cy, double rx, double ry, double phi, double x, double y) {
            return new float[] {
                (float) (cx + rx * Math.cos(phi) * x - ry * Math.sin(phi) * y),
                (float) (cy + rx * Math.sin(phi) * x + ry * Math.cos(phi) * y)
            };
        }

        private static double vectorAngle(double ux, double uy, double vx, double vy) {
            double length = Math.sqrt((ux * ux + uy * uy) * (vx * vx + vy * vy));
            if (length == 0) {
                return 0;
            }
            double value = Math.max(-1, Math.min(1, (ux * vx + uy * vy) / length));
            double sign = ux * vy - uy * vx < 0 ? -1 : 1;
            return sign * Math.acos(value);
        }
    }

    private static final class DoweSvgView extends View {
        private final float minX;
        private final float minY;
        private final float viewBoxWidth;
        private final float viewBoxHeight;
        private final int currentColor;
        private final ArrayList<DoweSvgPathEntry> paths;
        private final Paint paint = new Paint(Paint.ANTI_ALIAS_FLAG);

        DoweSvgView(Context context, float minX, float minY, float viewBoxWidth, float viewBoxHeight, int currentColor, ArrayList<DoweSvgPathEntry> paths) {
            super(context);
            this.minX = minX;
            this.minY = minY;
            this.viewBoxWidth = viewBoxWidth;
            this.viewBoxHeight = viewBoxHeight;
            this.currentColor = currentColor;
            this.paths = paths;
            paint.setStyle(Paint.Style.FILL);
            setLayoutParams(new LinearLayout.LayoutParams(ViewGroup.LayoutParams.WRAP_CONTENT, ViewGroup.LayoutParams.WRAP_CONTENT));
        }

        @Override
        protected void onDraw(Canvas canvas) {
            super.onDraw(canvas);
            float scaleX = getWidth() / viewBoxWidth;
            float scaleY = getHeight() / viewBoxHeight;
            Matrix matrix = new Matrix();
            matrix.postTranslate(-minX, -minY);
            matrix.postScale(scaleX, scaleY);
            for (DoweSvgPathEntry entry : paths) {
                Integer fill = entry.currentColor ? Integer.valueOf(currentColor) : entry.color;
                if (fill == null) {
                    continue;
                }
                Path path = DoweSvgPathParser.parse(entry.data);
                path.transform(matrix);
                paint.setColor(fill);
                canvas.drawPath(path, paint);
            }
        }
    }

    private DoweGridLayout doweGrid(Integer columns, Integer rowGap, Integer columnGap) {
        DoweGridLayout view = new DoweGridLayout(
            this,
            columns == null ? 1 : columns,
            doweDp(rowGap == null ? 0 : rowGap),
            doweDp(columnGap == null ? 0 : columnGap)
        );
        view.setLayoutParams(new LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT));
        return view;
    }

    private GradientDrawable doweBackground(int color, float radius) {
        GradientDrawable background = new GradientDrawable();
        background.setColor(color);
        background.setCornerRadius(doweDp(radius));
        return background;
    }

    private GradientDrawable doweSectionBackground(String value) {
        int[] colors;
        if ("aurora".equals(value)) {
            colors = new int[] { DOWE_SOFT_PRIMARY, DOWE_SOFT_SECONDARY, DOWE_SOFT_TERTIARY };
        } else if ("sunrise".equals(value)) {
            colors = new int[] { DOWE_SOFT_WARNING, DOWE_SOFT_DANGER, DOWE_SURFACE };
        } else if ("ocean".equals(value)) {
            colors = new int[] { DOWE_SOFT_INFO, DOWE_SOFT_PRIMARY, DOWE_SOFT_TERTIARY };
        } else if ("meadow".equals(value)) {
            colors = new int[] { DOWE_SOFT_SUCCESS, DOWE_SOFT_TERTIARY, DOWE_SURFACE };
        } else if ("slate".equals(value)) {
            colors = new int[] { DOWE_SOFT_MUTED, DOWE_SURFACE, DOWE_BACKGROUND };
        } else {
            colors = new int[] { DOWE_SURFACE, DOWE_BACKGROUND };
        }
        GradientDrawable background = new GradientDrawable(GradientDrawable.Orientation.TL_BR, colors);
        background.setCornerRadius(0);
        return background;
    }

    private GradientDrawable doweInputBackground(int color, Integer strokeColor, float radius) {
        GradientDrawable background = doweBackground(color, radius);
        if (strokeColor != null) {
            background.setStroke(doweDp(1), strokeColor);
        }
        return background;
    }

    private GradientDrawable doweDrawerBackground(int color, Integer strokeColor, String position, float radius) {
        GradientDrawable background = new GradientDrawable();
        background.setColor(color);
        float value = doweDp(radius);
        boolean rtl = getResources().getConfiguration().getLayoutDirection() == View.LAYOUT_DIRECTION_RTL;
        boolean attachedLeft = "start".equals(position) && !rtl || "end".equals(position) && rtl;
        if ("top".equals(position)) {
            background.setCornerRadii(new float[] { 0, 0, 0, 0, value, value, value, value });
        } else if ("bottom".equals(position)) {
            background.setCornerRadii(new float[] { value, value, value, value, 0, 0, 0, 0 });
        } else if (attachedLeft) {
            background.setCornerRadii(new float[] { 0, 0, value, value, value, value, 0, 0 });
        } else {
            background.setCornerRadii(new float[] { value, value, 0, 0, 0, 0, value, value });
        }
        if (strokeColor != null) {
            background.setStroke(doweDp(1), strokeColor);
        }
        return background;
    }

    private FrameLayout doweVideo(String source, String poster, boolean autoplay, String aspect, int backgroundColor, Integer borderColor) {
        DoweVideoLayout view = new DoweVideoLayout(this, doweVideoAspect(aspect));
        view.setBackground(borderColor == null ? doweBackground(backgroundColor, DOWE_RADIUS_BOX) : doweInputBackground(backgroundColor, borderColor, DOWE_RADIUS_BOX));
        VideoView video = new VideoView(this);
        MediaController controls = new MediaController(this);
        controls.setAnchorView(video);
        video.setMediaController(controls);
        video.setVideoURI(Uri.parse(source));
        view.addView(video, new FrameLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.MATCH_PARENT));
        ImageView posterView = poster == null ? null : new ImageView(this);
        if (posterView != null) {
            posterView.setScaleType(ImageView.ScaleType.CENTER_CROP);
            posterView.setImageURI(Uri.parse(poster));
            posterView.setOnClickListener(target -> {
                view.removeView(posterView);
                video.start();
            });
            view.addView(posterView, new FrameLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.MATCH_PARENT));
        }
        video.setOnPreparedListener(player -> {
            if (autoplay) {
                if (posterView != null) {
                    view.removeView(posterView);
                }
                video.start();
            }
        });
        return view;
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

    private static final class DoweVideoLayout extends FrameLayout {
        private final float aspect;

        DoweVideoLayout(Context context, float aspect) {
            super(context);
            this.aspect = aspect;
        }

        @Override
        protected void onMeasure(int widthSpec, int heightSpec) {
            int width = MeasureSpec.getSize(widthSpec);
            int height = Math.round(width / aspect);
            super.onMeasure(widthSpec, MeasureSpec.makeMeasureSpec(height, MeasureSpec.EXACTLY));
        }
    }

    private DoweCandlestickView doweCandlestick(String dataPath, String stream, int upColor, int downColor, String emptyLabel, int maxPoints, int backgroundColor, int contentColor, Integer borderColor) {
        DoweCandlestickView view = new DoweCandlestickView(this, dataPath, upColor, downColor, emptyLabel, maxPoints, contentColor);
        view.setBackground(borderColor == null ? doweInputBackground(backgroundColor, doweAlpha(contentColor, 0.12f), DOWE_RADIUS_BOX) : doweInputBackground(backgroundColor, borderColor, DOWE_RADIUS_BOX));
        view.setMinimumHeight(doweDp(220));
        view.setLayoutParams(new LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, doweDp(220)));
        view.startStream(stream);
        return view;
    }

    private String doweCandlestickStreamUrl(String stream) {
        if (stream == null || stream.isEmpty()) {
            return null;
        }
        if (stream.startsWith("https://")) {
            return stream;
        }
        if (stream.startsWith("/")) {
            String base = DoweEnvironment.BACKEND_URL.replaceAll("/+$", "");
            return base.isEmpty() ? null : base + stream;
        }
        return null;
    }

    private String doweCandlestickStreamPayload(String line) {
        String text = line == null ? "" : line.trim();
        return text.startsWith("data:") ? text.substring(5).trim() : text;
    }

    private Object doweCandlestickJson(String text) {
        try {
            if (text.startsWith("[")) {
                return doweFromJson(new JSONArray(text));
            }
            if (text.startsWith("{")) {
                return doweFromJson(new JSONObject(text));
            }
        } catch (Exception error) {
            return null;
        }
        return null;
    }

    private final class DoweCandlestickView extends View {
        private final String dataPath;
        private final int upColor;
        private final int downColor;
        private final String emptyLabel;
        private final int maxPoints;
        private final int contentColor;
        private final Paint paint = new Paint(Paint.ANTI_ALIAS_FLAG);
        private Thread streamThread;

        DoweCandlestickView(Context context, String dataPath, int upColor, int downColor, String emptyLabel, int maxPoints, int contentColor) {
            super(context);
            this.dataPath = dataPath;
            this.upColor = upColor;
            this.downColor = downColor;
            this.emptyLabel = emptyLabel;
            this.maxPoints = maxPoints;
            this.contentColor = contentColor;
        }

        private void startStream(String stream) {
            String address = doweCandlestickStreamUrl(stream);
            if (address == null) {
                return;
            }
            streamThread = new Thread(() -> {
                try {
                    HttpURLConnection connection = (HttpURLConnection) new URL(address).openConnection();
                    connection.setRequestProperty("Accept", "text/event-stream");
                    try (BufferedReader reader = new BufferedReader(new InputStreamReader(connection.getInputStream()))) {
                        String line;
                        while (!Thread.currentThread().isInterrupted() && (line = reader.readLine()) != null) {
                            String payloadText = doweCandlestickStreamPayload(line);
                            if (payloadText.isEmpty()) {
                                continue;
                            }
                            if ("[DONE]".equals(payloadText)) {
                                break;
                            }
                            Object payload = doweCandlestickJson(payloadText);
                            if (payload != null) {
                                runOnUiThread(() -> {
                                    doweUpsertCandles(dataPath, payload, maxPoints);
                                    invalidate();
                                });
                            }
                        }
                    }
                } catch (Exception error) {
                }
            });
            streamThread.start();
        }

        @Override
        protected void onDetachedFromWindow() {
            if (streamThread != null) {
                streamThread.interrupt();
            }
            super.onDetachedFromWindow();
        }

        @Override
        protected void onDraw(Canvas canvas) {
            super.onDraw(canvas);
            ArrayList<Map<String, Object>> source = doweCandles(dataPath);
            ArrayList<Map<String, Object>> candles = new ArrayList<>();
            int start = Math.max(0, source.size() - maxPoints);
            float high = -Float.MAX_VALUE;
            float low = Float.MAX_VALUE;
            for (int index = start; index < source.size(); index += 1) {
                Map<String, Object> candle = source.get(index);
                if (!doweValidCandle(candle)) {
                    continue;
                }
                Float candleHigh = doweCandleNumber(candle.get("high"));
                Float candleLow = doweCandleNumber(candle.get("low"));
                candles.add(candle);
                high = Math.max(high, candleHigh);
                low = Math.min(low, candleLow);
            }
            if (candles.isEmpty()) {
                paint.setColor(doweAlpha(contentColor, 0.64f));
                paint.setTextAlign(Paint.Align.CENTER);
                paint.setTextSize(13f * getResources().getDisplayMetrics().scaledDensity);
                canvas.drawText(emptyLabel, getWidth() / 2f, getHeight() / 2f, paint);
                return;
            }
            float top = doweDp(12f);
            float right = doweDp(12f);
            float bottom = doweDp(18f);
            float left = doweDp(12f);
            float width = Math.max(1f, getWidth() - left - right);
            float height = Math.max(1f, getHeight() - top - bottom);
            float range = Math.max(high - low, 0.000001f);
            float step = width / Math.max(candles.size(), 1);
            float bodyWidth = Math.max(doweDp(3f), Math.min(doweDp(12f), step * 0.56f));
            paint.setStrokeWidth(doweDp(1f));
            paint.setColor(doweAlpha(contentColor, 0.1f));
            for (int line = 0; line <= 3; line += 1) {
                float y = top + height * line / 3f;
                canvas.drawLine(left, y, left + width, y, paint);
            }
            for (int index = 0; index < candles.size(); index += 1) {
                Map<String, Object> candle = candles.get(index);
                float open = doweCandleNumber(candle.get("open"));
                float candleHigh = doweCandleNumber(candle.get("high"));
                float candleLow = doweCandleNumber(candle.get("low"));
                float close = doweCandleNumber(candle.get("close"));
                float centerX = left + step * (index + 0.5f);
                float highY = top + height * ((high - candleHigh) / range);
                float lowY = top + height * ((high - candleLow) / range);
                float openY = top + height * ((high - open) / range);
                float closeY = top + height * ((high - close) / range);
                int color = close >= open ? upColor : downColor;
                paint.setColor(color);
                paint.setStrokeWidth(doweDp(1.4f));
                canvas.drawLine(centerX, highY, centerX, lowY, paint);
                paint.setStyle(Paint.Style.FILL);
                float bodyTop = Math.min(openY, closeY);
                float bodyHeight = Math.max(doweDp(1f), Math.abs(closeY - openY));
                canvas.drawRoundRect(centerX - bodyWidth / 2f, bodyTop, centerX + bodyWidth / 2f, bodyTop + bodyHeight, doweDp(1.5f), doweDp(1.5f), paint);
            }
            paint.setStyle(Paint.Style.FILL);
        }
    }

    private LinearLayout doweCode(String source, String language, String[] tokenTexts, int[] tokenColors, String copyLabel, String copiedLabel, int backgroundColor, int contentColor, Integer borderColor) {
        LinearLayout view = doweContainer(false);
        view.setBackground(borderColor == null ? doweBackground(backgroundColor, DOWE_RADIUS_BOX) : doweInputBackground(backgroundColor, borderColor, DOWE_RADIUS_BOX));
        LinearLayout toolbar = doweContainer(true);
        toolbar.setGravity(Gravity.CENTER_VERTICAL);
        TextView languageView = doweText(language.toUpperCase(), contentColor, 12f, 700, 0.08f, 1.2f, "monospace");
        doweAdd(toolbar, languageView);
        View spacer = new View(this);
        spacer.setLayoutParams(new LinearLayout.LayoutParams(0, 1, 1f));
        toolbar.addView(spacer);
        Button copy = new Button(this);
        copy.setText(copyLabel);
        copy.setAllCaps(false);
        copy.setTextColor(contentColor);
        copy.setBackgroundColor(Color.TRANSPARENT);
        copy.setOnClickListener(target -> {
            ClipboardManager clipboard = (ClipboardManager) getSystemService(Context.CLIPBOARD_SERVICE);
            clipboard.setPrimaryClip(ClipData.newPlainText("code", source));
            copy.setText(copiedLabel);
            new Handler(Looper.getMainLooper()).postDelayed(() -> copy.setText(copyLabel), 1500);
        });
        doweAdd(toolbar, copy);
        toolbar.setPadding(doweDp(12), doweDp(6), doweDp(8), doweDp(6));
        doweAdd(view, toolbar);
        SpannableString highlighted = new SpannableString(source);
        int offset = 0;
        for (int index = 0; index < tokenTexts.length; index += 1) {
            int end = offset + tokenTexts[index].length();
            highlighted.setSpan(new ForegroundColorSpan(tokenColors[index]), offset, end, 0);
            offset = end;
        }
        TextView code = doweText(source, contentColor, 14f, 400, 0f, 1.6f, "monospace");
        code.setText(highlighted);
        code.setPadding(doweDp(16), doweDp(12), doweDp(16), doweDp(12));
        HorizontalScrollView scroll = new HorizontalScrollView(this);
        scroll.addView(code);
        doweAdd(view, scroll);
        return view;
    }

    private TextView doweText(String value, int color, float size, int weight, float letterSpacing, float lineHeight, String font) {
        TextView view = new TextView(this);
        view.setText(value);
        view.setTextColor(color);
        view.setTextSize(size);
        Typeface baseTypeface = Typeface.create(font, Typeface.NORMAL);
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.P) {
            view.setTypeface(Typeface.create(baseTypeface, weight, false));
        } else {
            view.setTypeface(Typeface.create(baseTypeface, weight >= 600 ? Typeface.BOLD : Typeface.NORMAL));
        }
        view.setLetterSpacing(letterSpacing);
        view.setLineSpacing(0f, lineHeight);
        view.setIncludeFontPadding(false);
        return view;
    }

    private TextView doweControlLabel(String value, int color, String font) {
        TextView view = doweText(value, color, 14f, 700, 0f, 1.2f, font);
        view.setGravity(Gravity.START);
        return view;
    }

    private FrameLayout doweFloatingInput(EditText input, String label, String placeholder, int color, String font, GradientDrawable background) {
        FrameLayout view = doweFloatingControl(background);
        FrameLayout.LayoutParams inputParams = new FrameLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT, Gravity.CENTER_VERTICAL);
        view.addView(input, inputParams);
        TextView labelView = doweControlLabel(label, color, font);
        view.addView(labelView);
        doweUpdateFloatingInputLabel(input, labelView, placeholder, color);
        input.setOnFocusChangeListener((target, focused) -> doweUpdateFloatingInputLabel(input, labelView, placeholder, color));
        input.addTextChangedListener(new TextWatcher() {
            public void beforeTextChanged(CharSequence value, int start, int count, int after) {}
            public void onTextChanged(CharSequence value, int start, int before, int count) {}
            public void afterTextChanged(Editable value) { doweUpdateFloatingInputLabel(input, labelView, placeholder, color); }
        });
        return view;
    }

    private TextView doweSelectTrigger(String placeholder, int color, String font) {
        TextView view = doweText(placeholder, color, 16f, 400, 0f, 1.25f, font);
        view.setGravity(Gravity.CENTER_VERTICAL | Gravity.START);
        view.setSingleLine(true);
        view.setClickable(true);
        view.setFocusable(true);
        return view;
    }

    private void doweBindSelect(TextView input, TextView floatingLabel, String[] labels, String[] values, String[] descriptions, String[] selected, String placeholder, int color, String font, String bindPath, boolean floating) {
        doweUpdateSelectTrigger(input, floatingLabel, labels, values, selected[0], placeholder, floating, false);
        input.setOnClickListener(view -> doweSelectPopup(input, floatingLabel, labels, values, descriptions, selected, placeholder, color, font, bindPath, floating));
    }

    private void doweUpdateSelectTrigger(TextView input, TextView floatingLabel, String[] labels, String[] values, String selected, String placeholder, boolean floating, boolean expanded) {
        String label = "";
        for (int i = 0; i < values.length; i++) {
            if (values[i].equals(selected)) {
                label = labels[i];
                break;
            }
        }
        boolean hasSelection = !label.isEmpty();
        if (hasSelection) {
            input.setText(label);
        } else if (!floating || expanded) {
            input.setText(placeholder);
        } else {
            input.setText("");
        }
        doweUpdateFloatingSelectLabel(input, floatingLabel, floating, expanded || hasSelection);
    }

    private void doweSelectPopup(TextView anchor, TextView floatingLabel, String[] labels, String[] values, String[] descriptions, String[] selected, String placeholder, int color, String font, String bindPath, boolean floating) {
        doweUpdateSelectTrigger(anchor, floatingLabel, labels, values, selected[0], placeholder, floating, true);
        LinearLayout content = doweContainer(false);
        content.setAlpha(0f);
        content.setScaleX(0.98f);
        content.setScaleY(0.98f);
        content.setTranslationY(-doweDp(4));
        content.setPadding(0, doweDp(4), 0, doweDp(4));
        content.setBackground(doweInputBackground(DOWE_SURFACE, doweAlpha(DOWE_ON_SURFACE, 0.08f), DOWE_RADIUS_UI));
        PopupWindow popup = new PopupWindow(content, Math.max(anchor.getWidth(), doweDp(220)), ViewGroup.LayoutParams.WRAP_CONTENT, true);
        popup.setOutsideTouchable(true);
        popup.setBackgroundDrawable(new android.graphics.drawable.ColorDrawable(Color.TRANSPARENT));
        popup.setOnDismissListener(() -> doweUpdateSelectTrigger(anchor, floatingLabel, labels, values, selected[0], placeholder, floating, false));
        for (int i = 0; i < labels.length; i++) {
            final int index = i;
            LinearLayout option = doweContainer(false);
            option.setPadding(doweDp(16), doweDp(10), doweDp(16), doweDp(10));
            if (values[index].equals(selected[0])) {
                option.setBackgroundColor(doweAlpha(color, 0.08f));
            }
            TextView labelView = doweText(labels[index], DOWE_ON_SURFACE, 16f, 700, 0f, 1.2f, font);
            doweAdd(option, labelView);
            if (!descriptions[index].isEmpty()) {
                TextView descriptionView = doweText(descriptions[index], doweAlpha(DOWE_ON_SURFACE, 0.68f), 12f, 400, 0f, 1.2f, font);
                doweAdd(option, descriptionView, 4, false);
            }
            option.setOnClickListener(view -> {
                selected[0] = values[index];
                doweUpdateSelectTrigger(anchor, floatingLabel, labels, values, selected[0], placeholder, floating, false);
                if (bindPath != null) {
                    doweWrite(bindPath, selected[0]);
                }
                popup.dismiss();
            });
            doweAdd(content, option);
        }
        popup.showAsDropDown(anchor, 0, doweDp(4));
        content.animate().alpha(1f).scaleX(1f).scaleY(1f).translationY(0f).setDuration(160).start();
    }

    private FrameLayout doweFloatingSelect(TextView input, TextView labelView, int color, GradientDrawable background) {
        FrameLayout view = doweSelectFrame(input, color, background);
        view.addView(labelView);
        return view;
    }

    private void doweUpdateFloatingSelectLabel(TextView input, TextView label, boolean floating, boolean active) {
        if (!floating || label == null) {
            return;
        }
        float baseSize = input.getTextSize() / getResources().getDisplayMetrics().scaledDensity;
        label.setTextSize(active ? 12f : baseSize);
        FrameLayout.LayoutParams labelParams = new FrameLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT, Gravity.START | (active ? Gravity.TOP : Gravity.CENTER_VERTICAL));
        labelParams.leftMargin = doweDp(12);
        labelParams.rightMargin = doweDp(36);
        labelParams.topMargin = active ? doweDp(2) : 0;
        label.setLayoutParams(labelParams);
        input.setPadding(input.getPaddingLeft(), active ? doweDp(10) : 0, input.getPaddingRight(), input.getPaddingBottom());
    }

    private FrameLayout doweSelectFrame(TextView input, int color, GradientDrawable background) {
        FrameLayout view = doweFloatingControl(background);
        FrameLayout.LayoutParams inputParams = new FrameLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.MATCH_PARENT, Gravity.CENTER_VERTICAL);
        view.addView(input, inputParams);
        view.addView(doweSelectArrow(color));
        return view;
    }

    private DoweSvgView doweSelectArrow(int color) {
        ArrayList<DoweSvgPathEntry> paths = new ArrayList<>();
        paths.add(new DoweSvgPathEntry("M0 0h24v24H0z", false, null));
        paths.add(new DoweSvgPathEntry("M19.716 13.705a1 1 0 0 0-1.425-1.404l-5.29 5.37V4a1 1 0 1 0-2 0v13.665L5.714 12.3a1 1 0 0 0-1.424 1.403l6.822 6.925a1.25 1.25 0 0 0 1.78 0z", true, null));
        DoweSvgView view = new DoweSvgView(this, 0f, 0f, 24f, 24f, color, paths);
        FrameLayout.LayoutParams params = new FrameLayout.LayoutParams(doweDp(16), doweDp(16), Gravity.END | Gravity.CENTER_VERTICAL);
        params.rightMargin = doweDp(12);
        view.setLayoutParams(params);
        view.setImportantForAccessibility(View.IMPORTANT_FOR_ACCESSIBILITY_NO);
        return view;
    }

    private FrameLayout doweFloatingControl(GradientDrawable background) {
        FrameLayout view = new FrameLayout(this);
        view.setLayoutParams(new LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT));
        view.setMinimumHeight(doweDp(40));
        view.setBackground(background);
        return view;
    }

    private void doweUpdateFloatingInputLabel(EditText input, TextView label, String placeholder, int color) {
        boolean active = input.hasFocus() || input.getText().length() > 0;
        float baseSize = input.getTextSize() / getResources().getDisplayMetrics().scaledDensity;
        label.setTextSize(active ? 12f : baseSize);
        FrameLayout.LayoutParams labelParams = new FrameLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT, Gravity.START | (active ? Gravity.TOP : Gravity.CENTER_VERTICAL));
        labelParams.leftMargin = doweDp(12);
        labelParams.rightMargin = doweDp(12);
        labelParams.topMargin = active ? doweDp(2) : 0;
        label.setLayoutParams(labelParams);
        input.setHint(active ? placeholder : "");
        input.setHintTextColor(doweAlpha(color, 0.55f));
    }

    private int doweAlpha(int color, float alpha) {
        return Color.argb(Math.round(Color.alpha(color) * alpha), Color.red(color), Color.green(color), Color.blue(color));
    }

    private void doweAdd(ViewGroup parent, View child) {
        parent.addView(child);
    }

    private void doweAdd(ViewGroup parent, View child, Integer gap, boolean horizontal) {
        if (gap != null && parent.getChildCount() > 0) {
            ViewGroup.LayoutParams current = child.getLayoutParams();
            LinearLayout.LayoutParams params;
            if (current instanceof LinearLayout.LayoutParams) {
                params = new LinearLayout.LayoutParams((LinearLayout.LayoutParams) current);
            } else if (current != null) {
                params = new LinearLayout.LayoutParams(current.width, current.height);
            } else {
                params = new LinearLayout.LayoutParams(ViewGroup.LayoutParams.WRAP_CONTENT, ViewGroup.LayoutParams.WRAP_CONTENT);
            }
            int size = doweDp(gap);
            if (horizontal) {
                params.setMargins(size, 0, 0, 0);
            } else {
                params.setMargins(0, size, 0, 0);
            }
            child.setLayoutParams(params);
        }
        parent.addView(child);
    }

__DOWE_JAVA_REACTIVE_RUNTIME__
    private Integer doweResponsiveInt(int viewportWidth, Integer xs, Integer sm, Integer md, Integer lg, Integer xl) {
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
"#,
    );
    output = output.replace(
        "__DOWE_DEFAULT_FONT__",
        font_config
            .default_family
            .catalog_entry()
            .android_family_name,
    );
    output = output.replace("__DOWE_JAVA_REACTIVE_RUNTIME__", dev_java_reactive_runtime());
    output
}
