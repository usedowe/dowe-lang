function treeChildren(value) {
  if (Array.isArray(value)) return value;
  if (!value || typeof value !== "object") return [];
  if (Array.isArray(value.children)) return value.children;
  const folders = Array.isArray(value.folders) ? value.folders : [];
  const files = Array.isArray(value.files) ? value.files : [];
  return folders.concat(files);
}
function treeNode(value) {
  if (!value || typeof value !== "object" || Array.isArray(value)) return null;
  const rawId = value.id ?? value.path ?? value.name ?? value.label;
  if (rawId == null || String(rawId) === "") return null;
  const id = String(rawId);
  const rawLabel = value.name ?? value.label;
  if (rawLabel == null || String(rawLabel) === "") return null;
  const label = String(rawLabel);
  const path = String(value.path ?? id);
  const childValues = treeChildren(value);
  const explicitChildren =
    Array.isArray(value.children) ||
    Array.isArray(value.folders) ||
    Array.isArray(value.files);
  const kind = String(value.type ?? value.kind ?? "").toLowerCase();
  const branch =
    childValues.length > 0 ||
    explicitChildren ||
    kind === "folder" ||
    kind === "directory" ||
    kind === "branch";
  return { id, label, path, branch, children: null, value };
}
function treeNodeChildren(node) {
  if (node.children) return node.children;
  node.children = treeChildren(node.value).map(treeNode).filter(Boolean);
  return node.children;
}
function treeNodes(value) {
  return treeChildren(value).map(treeNode).filter(Boolean);
}
function treeIconTemplate(tree, name) {
  return tree.querySelector(`[data-dowe-tree-icon="${name}"]`);
}
function treeIcon(tree, name) {
  const template = treeIconTemplate(tree, name);
  return template?.content?.firstElementChild?.cloneNode(true) || null;
}
function treeOpen(tree, id) {
  if (!tree.__doweTreeOpen) tree.__doweTreeOpen = new Map();
  if (!tree.__doweTreeOpen.has(id)) {
    tree.__doweTreeOpen.set(
      id,
      tree.dataset.doweTreeDefaultOpen !== "false"
    );
  }
  return tree.__doweTreeOpen.get(id);
}
function treeSetOpen(tree, id, open) {
  if (!tree.__doweTreeOpen) tree.__doweTreeOpen = new Map();
  tree.__doweTreeOpen.set(id, open);
}
function treeNodePath(node) {
  return node.path || node.id;
}
function treeIndent(depth) {
  const indent = document.createDocumentFragment();
  for (let index = 0; index < depth; index += 1) {
    const guide = document.createElement("span");
    guide.className = "tree-indent";
    guide.setAttribute("aria-hidden", "true");
    indent.appendChild(guide);
  }
  return indent;
}
function renderTreeNode(tree, node, depth, selected) {
  const wrapper = document.createElement("div");
  wrapper.className = "tree-node";
  wrapper.dataset.doweTreeNode = "";
  wrapper.dataset.doweTreeNodeId = node.id;
  wrapper.dataset.doweTreeNodePath = treeNodePath(node);
  wrapper.dataset.doweTreeNodeKind = node.branch ? "branch" : "leaf";
  wrapper.__doweTreeItem = node.value;
  tree.__doweTreeRows.set(treeNodePath(node), wrapper);

  const row = document.createElement("div");
  row.className = "tree-row";
  row.classList.toggle("is-selected", !node.branch && selected === node.path);
  row.appendChild(treeIndent(depth));

  if (node.branch) {
    const open = treeOpen(tree, node.id);
    const toggle = document.createElement("button");
    toggle.className = "tree-toggle";
    toggle.type = "button";
    toggle.dataset.doweTreeToggle = "";
    toggle.setAttribute("aria-label", `${open ? "Collapse" : "Expand"} ${node.label}`);
    toggle.setAttribute("aria-expanded", open ? "true" : "false");
    const arrow = treeIcon(tree, "arrow");
    if (arrow) toggle.appendChild(arrow);
    row.appendChild(toggle);
  } else {
    const placeholder = document.createElement("span");
    placeholder.className = "tree-toggle-placeholder";
    placeholder.setAttribute("aria-hidden", "true");
    row.appendChild(placeholder);
  }

  const item = document.createElement("button");
  item.className = `tree-item ${node.branch ? "tree-branch" : "tree-leaf"}`;
  item.type = "button";
  item.setAttribute("role", "treeitem");
  item.setAttribute("aria-selected", !node.branch && selected === node.path ? "true" : "false");
  if (node.branch) {
    item.dataset.doweTreeBranch = "";
    item.setAttribute("aria-expanded", treeOpen(tree, node.id) ? "true" : "false");
  } else {
    item.dataset.doweTreeSelect = "";
  }
  const icon = treeIcon(tree, node.branch ? "folder" : "file");
  if (icon) {
    const iconWrap = document.createElement("span");
    iconWrap.className = "tree-icon";
    iconWrap.setAttribute("aria-hidden", "true");
    iconWrap.appendChild(icon);
    item.appendChild(iconWrap);
  }
  const label = document.createElement("span");
  label.className = "tree-label";
  label.textContent = node.label;
  item.appendChild(label);
  row.appendChild(item);
  wrapper.appendChild(row);

  if (node.branch) {
    const children = document.createElement("div");
    children.className = "tree-children";
    children.setAttribute("role", "group");
    const open = treeOpen(tree, node.id);
    children.hidden = !open;
    if (open) {
      for (const child of treeNodeChildren(node)) {
        children.appendChild(renderTreeNode(tree, child, depth + 1, selected));
      }
    }
    wrapper.appendChild(children);
  }
  return wrapper;
}
function updateTreeRowSelection(row, selected) {
  const line = row?.firstElementChild;
  if (!line) return;
  line.classList.toggle("is-selected", selected);
  const item = line.querySelector("[data-dowe-tree-select]");
  if (item) item.setAttribute("aria-selected", selected ? "true" : "false");
}
function updateTreeSelection(tree, selected) {
  const value = String(selected || "");
  const previous = tree.__doweTreeSelected || "";
  if (previous === value) return;
  updateTreeRowSelection(tree.__doweTreeRows?.get(previous), false);
  updateTreeRowSelection(tree.__doweTreeRows?.get(value), true);
  tree.__doweTreeSelected = value;
  tree.__doweTreeRenderedSelected = value;
}
function renderTree(tree, state, scope) {
  const content = tree.querySelector("[data-dowe-tree-content]");
  const empty = tree.querySelector("[data-dowe-tree-empty]");
  if (!content || !empty) return;
  const data = readPath(state, tree.dataset.doweTreeData, scope);
  const bound = tree.dataset.doweTreeBind;
  const selected = bound
    ? String(readPath(state, bound, scope) ?? "")
    : tree.__doweTreeSelected || "";
  if (
    !tree.__doweTreeDirty &&
    tree.__doweTreeRenderedData === data &&
    tree.__doweTreeRenderedSelected === selected
  )
    return;
  const nodes =
    tree.__doweTreeNodesData === data && tree.__doweTreeNodes
      ? tree.__doweTreeNodes
      : treeNodes(data);
  content.replaceChildren();
  tree.__doweTreeRows = new Map();
  tree.__doweTreeNodesData = data;
  tree.__doweTreeNodes = nodes;
  tree.__doweTreeSelected = selected;
  tree.__doweTreeRenderedData = data;
  tree.__doweTreeRenderedSelected = selected;
  tree.__doweTreeDirty = false;
  empty.hidden = nodes.length > 0;
  if (!nodes.length) {
    empty.textContent = tree.dataset.doweTreeEmptyLabel || "No files";
    return;
  }
  for (const node of nodes) {
    content.appendChild(renderTreeNode(tree, node, 0, selected));
  }
}
function treeActionScope(tree, item) {
  return { ...(scopeFor(tree) || {}), item };
}
function toggleTreeNode(tree, node) {
  const open = !treeOpen(tree, node.id);
  treeSetOpen(tree, node.id, open);
  tree.__doweTreeDirty = true;
  renderTree(tree, activeView?.state || {}, scopeFor(tree));
}
function selectTreeNode(tree, node) {
  const value = treeNodePath(node);
  const bind = tree.dataset.doweTreeBind;
  if (bind && activeView) {
    writePath(activeView.state, bind, value);
  } else {
    tree.__doweTreeSelected = value;
  }
  updateTreeSelection(tree, value);
  const action = tree.dataset.doweTreeOnSelect;
  if (action) runAction(action, treeActionScope(tree, node.value));
}
function hydrateTree(tree) {
  if (tree.__doweTreeHydrated) return;
  tree.__doweTreeHydrated = true;
  tree.addEventListener("click", event => {
    const target = event.target;
    const toggle = target?.closest?.("[data-dowe-tree-toggle]");
    if (toggle && tree.contains(toggle)) {
      event.preventDefault();
      const node = toggle.closest("[data-dowe-tree-node]");
      if (node?.__doweTreeItem) {
        const record = treeNode(node.__doweTreeItem);
        if (record) toggleTreeNode(tree, record);
      }
      return;
    }
    const branch = target?.closest?.("[data-dowe-tree-branch]");
    if (branch && tree.contains(branch)) {
      event.preventDefault();
      const node = branch.closest("[data-dowe-tree-node]");
      if (node?.__doweTreeItem) {
        const record = treeNode(node.__doweTreeItem);
        if (record) toggleTreeNode(tree, record);
      }
      return;
    }
    const select = target?.closest?.("[data-dowe-tree-select]");
    if (!select || !tree.contains(select)) return;
    event.preventDefault();
    const node = select.closest("[data-dowe-tree-node]");
    if (node?.__doweTreeItem) {
      const record = treeNode(node.__doweTreeItem);
      if (record && !record.branch) selectTreeNode(tree, record);
    }
  });
}
function renderTrees(root, state, scope) {
  const scoped = !!scope;
  for (const tree of root.querySelectorAll("[data-dowe-tree]")) {
    if (!scoped && tree.closest("[data-dowe-each-row]")) continue;
    hydrateTree(tree);
    renderTree(tree, state, scope);
  }
}
