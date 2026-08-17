use std::cell::{Cell, RefCell};
use std::rc::Rc;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewInspectorLocation {
    pub path: String,
    pub line: usize,
    pub column: usize,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewInspectorProp {
    pub name: String,
    pub value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewInspectorSignal {
    pub id: String,
    pub name: String,
    pub scope: String,
    pub storage: String,
    pub initial_json: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewInspectorAction {
    pub id: String,
    pub name: String,
    pub kind: String,
    pub detail: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ViewInspectorNode {
    pub id: String,
    pub kind: String,
    pub source_path: String,
    pub start_line: usize,
    pub end_line: usize,
    pub usages: Vec<ViewInspectorLocation>,
    pub props: Vec<ViewInspectorProp>,
    pub signals: Vec<ViewInspectorSignal>,
    pub actions: Vec<ViewInspectorAction>,
}

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct ViewInspectorMap {
    pub nodes: Vec<ViewInspectorNode>,
}

impl ViewInspectorMap {
    pub fn is_empty(&self) -> bool {
        self.nodes.is_empty()
    }
}

#[derive(Debug)]
struct ViewInspectorCursor {
    nodes: Vec<ViewInspectorNode>,
    index: usize,
}

impl ViewInspectorCursor {
    fn next_id(&mut self) -> Option<String> {
        let node = self.nodes.get(self.index)?;
        self.index += 1;
        Some(node.id.clone())
    }
}

type SharedViewInspectorCursor = Rc<RefCell<ViewInspectorCursor>>;

thread_local! {
    static VIEW_INSPECTOR_CURSOR: RefCell<Option<SharedViewInspectorCursor>> = const { RefCell::new(None) };
    static VIEW_INSPECTOR_MARKER_AVAILABLE: Cell<bool> = const { Cell::new(false) };
}

fn with_view_inspector<T>(map: Option<&ViewInspectorMap>, render: impl FnOnce() -> T) -> T {
    let cursor = map.map(|map| {
        Rc::new(RefCell::new(ViewInspectorCursor {
            nodes: map.nodes.clone(),
            index: 0,
        }))
    });
    VIEW_INSPECTOR_CURSOR.with(|current| {
        *current.borrow_mut() = cursor;
    });
    let result = render();
    VIEW_INSPECTOR_CURSOR.with(|current| {
        *current.borrow_mut() = None;
    });
    VIEW_INSPECTOR_MARKER_AVAILABLE.with(|available| available.set(false));
    result
}

fn with_view_inspector_node<T>(render: impl FnOnce() -> T) -> T {
    let previous = VIEW_INSPECTOR_MARKER_AVAILABLE.with(|available| available.replace(true));
    let result = render();
    VIEW_INSPECTOR_MARKER_AVAILABLE.with(|available| available.set(previous));
    result
}

fn next_view_inspector_id() -> Option<String> {
    VIEW_INSPECTOR_CURSOR.with(|current| {
        current.borrow().as_ref().and_then(|cursor| {
            VIEW_INSPECTOR_MARKER_AVAILABLE.with(|available| {
                if !available.replace(false) {
                    return None;
                }
                cursor.borrow_mut().next_id()
            })
        })
    })
}
