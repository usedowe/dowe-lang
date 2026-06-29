fn dev_activity_layout_widgets() -> &'static str {
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
        final ArrayList<DoweSideNavEntry> children;

        DoweSideNavEntry(String id, String kind, String label, String description, String status, String operation, String path, String fragment, boolean open, ArrayList<DoweSideNavEntry> children) {
            this.id = id;
            this.kind = kind;
            this.label = label;
            this.description = description;
            this.status = status;
            this.operation = operation;
            this.path = path;
            this.fragment = fragment;
            this.open = open;
            this.children = children == null ? new ArrayList<>() : children;
        }
    }

    private void doweRenderSideNav(LinearLayout parent, ArrayList<DoweSideNavEntry> entries, int paddingHorizontal, int paddingVertical, int labelSize, int descriptionSize, int backgroundColor, int contentColor, String font) {
        for (DoweSideNavEntry entry : entries) {
            if ("divider".equals(entry.kind)) {
                View divider = new View(this);
                divider.setBackgroundColor(DOWE_MUTED);
                divider.setLayoutParams(new LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, doweDp(1)));
                doweAdd(parent, divider, 8, false);
            } else if ("submenu".equals(entry.kind)) {
                LinearLayout trigger = doweSideNavRow(entry, true, paddingHorizontal, paddingVertical, labelSize, descriptionSize, backgroundColor, contentColor, font, null);
                doweAdd(parent, trigger);
                LinearLayout submenu = doweContainer(false);
                submenu.setPadding(doweDp(16), 0, 0, 0);
                submenu.setVisibility(entry.open ? View.VISIBLE : View.GONE);
                doweAdd(parent, submenu);
                trigger.setOnClickListener(v -> doweToggleSideNavSubmenu(submenu));
                doweRenderSideNav(submenu, entry.children, paddingHorizontal, paddingVertical, labelSize, descriptionSize, backgroundColor, contentColor, font);
            } else {
                LinearLayout row = doweSideNavRow(entry, "header".equals(entry.kind), paddingHorizontal, paddingVertical, labelSize, descriptionSize, backgroundColor, contentColor, font, doweSideNavAction(entry));
                doweAdd(parent, row);
            }
        }
    }

    private LinearLayout doweSideNavRow(DoweSideNavEntry entry, boolean header, int paddingHorizontal, int paddingVertical, int labelSize, int descriptionSize, int backgroundColor, int contentColor, String font, Runnable action) {
        LinearLayout view = doweContainer(true);
        view.setGravity(Gravity.CENTER_VERTICAL);
        view.setPadding(doweDp(paddingHorizontal), doweDp(paddingVertical), doweDp(paddingHorizontal), doweDp(paddingVertical));
        if (entry.path != null && entry.path.equals(currentPath)) {
            view.setBackground(doweBackground(backgroundColor, DOWE_RADIUS_UI));
        }
        LinearLayout copy = doweContainer(false);
        copy.setLayoutParams(new LinearLayout.LayoutParams(0, ViewGroup.LayoutParams.WRAP_CONTENT, 1f));
        doweAdd(view, copy);
        TextView label = doweText(entry.label, contentColor, labelSize, header ? 600 : 400, 0f, labelSize, font);
        doweAdd(copy, label);
        if (entry.description != null) {
            TextView description = doweText(entry.description, contentColor, descriptionSize, 400, 0f, descriptionSize, font);
            description.setAlpha(0.72f);
            doweAdd(copy, description);
        }
        if (entry.status != null) {
            TextView status = doweText(entry.status, contentColor, descriptionSize, 600, 0f, descriptionSize, font);
            doweAdd(view, status);
        }
        if (action != null) {
            view.setOnClickListener(v -> action.run());
        }
        return view;
    }

    private Runnable doweSideNavAction(DoweSideNavEntry entry) {
        if (entry.path == null) {
            return null;
        }
        return () -> doweNavigate(entry.operation == null ? "push" : entry.operation, entry.path, entry.fragment);
    }

"#
}
