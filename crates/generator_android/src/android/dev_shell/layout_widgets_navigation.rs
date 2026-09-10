r#"        DoweTreeIcon(float minX, float minY, float width, float height, ArrayList<DoweSvgPathEntry> paths) {
            this.minX = minX;
            this.minY = minY;
            this.width = width;
            this.height = height;
            this.paths = paths;
        }
    }

    private static final class DoweTreeViewState {
        final String dataPath;
        final String bindPath;
        final boolean defaultOpen;
        final String emptyLabel;
        final String ariaLabel;
        final String onSelect;
        final int backgroundColor;
        final int contentColor;
        final Integer borderColor;
        final float radius;
        String selected;
        final DoweTreeIcon folderIcon;
        final DoweTreeIcon fileIcon;
        final DoweTreeIcon arrowIcon;
        final HashMap<String, Boolean> open = new HashMap<>();

        DoweTreeViewState(String dataPath, String bindPath, boolean defaultOpen, String emptyLabel, String ariaLabel, String onSelect, int backgroundColor, int contentColor, Integer borderColor, float radius, DoweTreeIcon folderIcon, DoweTreeIcon fileIcon, DoweTreeIcon arrowIcon) {
            this.dataPath = dataPath;
            this.bindPath = bindPath;
            this.defaultOpen = defaultOpen;
            this.emptyLabel = emptyLabel;
            this.ariaLabel = ariaLabel;
            this.onSelect = onSelect;
            this.backgroundColor = backgroundColor;
            this.contentColor = contentColor;
            this.borderColor = borderColor;
            this.radius = radius;
            this.selected = "";
            this.folderIcon = folderIcon;
            this.fileIcon = fileIcon;
            this.arrowIcon = arrowIcon;
        }
    }

    private DoweSvgView doweTreeIcon(DoweTreeIcon icon, int color) {
        DoweSvgView view = new DoweSvgView(this, icon.minX, icon.minY, icon.width, icon.height, color, icon.paths);
        view.setImportantForAccessibility(View.IMPORTANT_FOR_ACCESSIBILITY_NO);
        return view;
    }

    private LinearLayout doweTree(String dataPath, String bindPath, boolean defaultOpen, String emptyLabel, String ariaLabel, String onSelect, int backgroundColor, int contentColor, Integer borderColor, float radius, DoweTreeIcon folderIcon, DoweTreeIcon fileIcon, DoweTreeIcon arrowIcon) {
        LinearLayout view = doweContainer(false);
        view.setPadding(0, doweDp(4), 0, doweDp(4));
        view.setBackground(borderColor == null ? doweBackground(backgroundColor, radius) : doweInputBackground(backgroundColor, borderColor, radius));
        view.setContentDescription(ariaLabel);
        DoweTreeViewState tree = new DoweTreeViewState(dataPath, bindPath, defaultOpen, emptyLabel, ariaLabel, onSelect, backgroundColor, contentColor, borderColor, radius, folderIcon, fileIcon, arrowIcon);
        view.setTag(tree);
        doweTreeRender(view, tree);
        return view;
    }

    private void doweTreeRender(LinearLayout view, DoweTreeViewState tree) {
        view.removeAllViews();
        ArrayList<DoweTreeNode> nodes = doweTreeNodes(tree.dataPath);
        if (nodes.isEmpty()) {
            TextView empty = doweText(tree.emptyLabel, doweAlpha(tree.contentColor, 0.64f), 14f, 400, 0f, 1.2f, null);
            empty.setPadding(doweDp(12), doweDp(10), doweDp(12), doweDp(10));
            doweAdd(view, empty);
            return;
        }
        String selectionKey = doweTreeSelectionKey(tree);
        String selected = tree.bindPath == null ? doweTreeSelected.getOrDefault(selectionKey, tree.selected) : doweTextValue(tree.bindPath, null);
        tree.selected = selected;
        for (DoweTreeNode node : nodes) doweTreeRenderNode(view, tree, node, 0, selected);
    }

    private String doweTreeMemoryKey(DoweTreeViewState tree, DoweTreeNode node) {
        return currentPath + ":" + tree.dataPath + ":" + node.id;
    }

    private String doweTreeSelectionKey(DoweTreeViewState tree) {
        return currentPath + ":" + tree.dataPath;
    }

    private void doweTreeRenderNode(LinearLayout parent, DoweTreeViewState tree, DoweTreeNode node, int depth, String selected) {
        String memoryKey = doweTreeMemoryKey(tree, node);
        boolean open = tree.open.containsKey(node.id)
            ? tree.open.get(node.id)
            : doweTreeOpen.containsKey(memoryKey)
                ? doweTreeOpen.get(memoryKey)
                : tree.defaultOpen;
        tree.open.put(node.id, open);
        boolean isSelected = !node.branch && node.path.equals(selected);
        LinearLayout row = doweContainer(true);
        row.setMinimumHeight(doweDp(28));
        row.setGravity(Gravity.CENTER_VERTICAL);
        row.setPadding(doweDp(4), 0, doweDp(4), 0);
        row.setContentDescription(node.branch ? (open ? "Collapse " : "Expand ") + node.label : node.label);
        row.setFocusable(true);
        row.setBackground(doweBackground(isSelected ? doweAlpha(tree.contentColor, 0.14f) : Color.TRANSPARENT, Math.min(tree.radius, doweDp(5))));
        for (int index = 0; index < depth; index += 1) {
            View indent = new View(this);
            doweAdd(row, indent, 0, true);
            indent.setLayoutParams(new LinearLayout.LayoutParams(doweDp(16), doweDp(28)));
        }
        if (node.branch) {
            DoweSvgView toggle = doweTreeIcon(tree.arrowIcon, doweAlpha(tree.contentColor, 0.72f));
            toggle.setRotation(open ? 0f : -90f);
            toggle.setLayoutParams(new LinearLayout.LayoutParams(doweDp(20), doweDp(28)));
            doweAdd(row, toggle);
        } else {
            View toggle = new View(this);
            toggle.setLayoutParams(new LinearLayout.LayoutParams(doweDp(20), doweDp(28)));
            doweAdd(row, toggle);
        }
        DoweSvgView icon = doweTreeIcon(node.branch ? tree.folderIcon : tree.fileIcon, doweAlpha(tree.contentColor, node.branch ? 0.88f : 0.72f));
        icon.setLayoutParams(new LinearLayout.LayoutParams(doweDp(16), doweDp(28)));
        doweAdd(row, icon, 2, true);
        TextView label = doweText(node.label, tree.contentColor, 14f, node.branch ? 600 : 400, 0f, 1.2f, null);
        label.setSingleLine(true);
        label.setEllipsize(android.text.TextUtils.TruncateAt.END);
        label.setLayoutParams(new LinearLayout.LayoutParams(0, ViewGroup.LayoutParams.WRAP_CONTENT, 1f));
        doweAdd(row, label, 6, true);
        row.setOnClickListener(target -> {
            if (node.branch) {
                boolean nextOpen = !open;
                tree.open.put(node.id, nextOpen);
                doweTreeOpen.put(memoryKey, nextOpen);
                doweTreeRender(parent, tree);
            } else {
                if (tree.bindPath != null) {
                    doweWrite(tree.bindPath, node.path);
                } else {
                    tree.selected = node.path;
                    doweTreeSelected.put(doweTreeSelectionKey(tree), node.path);
                }
                if (tree.onSelect != null) doweRunAction(tree.onSelect, node.value);
                renderCurrentRoute(false);
            }
        });
        doweAdd(parent, row);
        if (node.branch && open) {
            for (DoweTreeNode child : node.children) doweTreeRenderNode(parent, tree, child, depth + 1, selected);
        }
    }

    private LinearLayout doweTable(String dataPath, String[] fields, String[] labels, int[] alignments, String[] widths, int tableSize, boolean striped, boolean bordered, boolean dividers, String emptyTitle, String emptyDescription, int backgroundColor, int contentColor, Integer borderColor) {
        LinearLayout view = doweContainer(false);
        view.setBackground(borderColor == null
            ? bordered
                ? doweInputBackground(backgroundColor, doweAlpha(DOWE_SURFACE_TEXT, 0.28f), DOWE_RADIUS)
                : doweBackground(backgroundColor, DOWE_RADIUS)
            : doweInputBackground(backgroundColor, borderColor, DOWE_RADIUS));
        HorizontalScrollView scroll = new HorizontalScrollView(this);
        scroll.setLayoutParams(new LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT));
        scroll.setFillViewport(true);
        LinearLayout table = doweContainer(false);
        table.setLayoutParams(new HorizontalScrollView.LayoutParams(ViewGroup.LayoutParams.WRAP_CONTENT, ViewGroup.LayoutParams.WRAP_CONTENT));
        table.setMinimumWidth(doweTableMinimumWidth(widths));
        LinearLayout header = doweTableRow();
        header.setBackgroundColor(DOWE_MUTED);
        for (int index = 0; index < labels.length; index += 1) {
            TextView cell = doweTableCell(labels[index], contentColor, tableSize, true, alignments[index], widths[index], false);
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
                    row.setBackgroundColor(doweAlpha(DOWE_SURFACE_TEXT, 0.12f));
                }
                for (int columnIndex = 0; columnIndex < fields.length; columnIndex += 1) {
                    boolean separated = bordered && columnIndex < fields.length - 1;
                    TextView cell = doweTableCell(doweTableValue(rows.get(rowIndex), fields[columnIndex]), contentColor, tableSize, false, alignments[columnIndex], widths[columnIndex], separated);
                    doweAdd(row, cell);
                    if (separated) {
                        doweAdd(row, doweTableSeparator());
                    }
                }
                doweAdd(table, row);
                if (dividers && rowIndex < rows.size() - 1) {
                    View divider = new View(this);
                    divider.setBackgroundColor(doweAlpha(DOWE_SURFACE_TEXT, 0.28f));
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
        row.setLayoutParams(new LinearLayout.LayoutParams(ViewGroup.LayoutParams.MATCH_PARENT, ViewGroup.LayoutParams.WRAP_CONTENT));
        return row;
    }

    private TextView doweTableCell(String value, int color, int tableSize, boolean header, int gravity, String width, boolean reserveSeparator) {
        float textSize = tableSize == 2 ? 16f : tableSize == 0 ? 12f : 14f;
        TextView cell = doweText(value, color, textSize, header ? 700 : 400, 0f, 1.25f, "sans");
        int horizontal = tableSize == 2 ? 20 : tableSize == 0 ? 12 : 16;
        int vertical = tableSize == 2 ? (header ? 16 : 20) : tableSize == 0 ? 8 : (header ? 12 : 16);
        cell.setGravity(gravity | Gravity.CENTER_VERTICAL);
        cell.setSingleLine(true);
        cell.setPadding(doweDp(horizontal), doweDp(vertical), doweDp(horizontal), doweDp(vertical));
        cell.setLayoutParams(new LinearLayout.LayoutParams(doweTableColumnWidth(width) - (reserveSeparator ? doweDp(1) : 0), ViewGroup.LayoutParams.WRAP_CONTENT, 1f));
        return cell;
    }

    private View doweTableSeparator() {
        View separator = new View(this);
        separator.setBackgroundColor(doweAlpha(DOWE_SURFACE_TEXT, 0.28f));
        separator.setLayoutParams(new LinearLayout.LayoutParams(doweDp(1), ViewGroup.LayoutParams.MATCH_PARENT));
        return separator;
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

    private int doweTableMinimumWidth(String[] widths) {
        int value = 0;
        for (String width : widths) {
            value += doweTableColumnWidth(width);
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
"#
