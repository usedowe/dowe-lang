use super::{format_approval, format_approval_batch, format_value};
use serde_json::json;

#[test]
fn formats_nested_objects_and_lists_as_labeled_lines() {
    let value = json!({
        "session": {
            "id": "abc",
            "active": true,
            "usage": {"input": 12, "output": null}
        },
        "events": [
            {"kind": "message", "text": "hello"},
            {"kind": "tool", "ok": false}
        ],
        "empty": []
    });

    assert_eq!(
        format_value(&value),
        "empty: (none)\nevents:\n  - kind: message\n    text: hello\n  - kind: tool\n    ok: false\nsession:\n  active: true\n  id: abc\n  usage:\n    input: 12\n    output: null"
    );
}

#[test]
fn formats_session_inventory_with_human_metadata() {
    let rendered = format_value(&json!({
        "inventory": [{
            "id": "0123456789abcdef",
            "title": "Build dashboard",
            "initial_prompt_preview": "Build a dashboard with safe colors"
        }]
    }));
    assert!(rendered.contains("title: Build dashboard"));
    assert!(rendered.contains("initial_prompt_preview: Build a dashboard with safe colors"));
    assert!(rendered.contains("id: 0123456789abcdef"));
}

#[test]
fn formats_sessions_with_active_marker_and_compact_entries() {
    let rendered = format_value(&json!({
        "sessions": ["0123456789abcdef", "fedcba9876543210"],
        "active": "0123456789abcdef",
        "inventory": [
            {"id": "0123456789abcdef", "title": "Build dashboard", "state": "recorded", "catalog_compatible": true},
            {"id": "fedcba9876543210", "title": null, "initial_prompt_preview": "Fallback prompt", "state": "interrupted", "catalog_compatible": false}
        ]
    }));

    assert_eq!(
        rendered,
        "Sessions (2)\nActive: 01234567 · Build dashboard\n*  1. Build dashboard                  01234567 ✓ \n   2. Fallback prompt                  fedcba98 ×!"
    );
}

#[test]
fn formats_sessions_truncate_titles_without_wrapping() {
    let rendered = format_value(&json!({
        "sessions": ["0123456789abcdef"],
        "active": "0123456789abcdef",
        "inventory": [{"id": "0123456789abcdef", "title": "A very long title that should be safely bounded so it never causes ugly terminal wrapping", "state": "recorded", "catalog_compatible": true}]
    }));
    let lines = rendered.lines().collect::<Vec<_>>();
    let entry = lines[2];

    assert!(lines[1].chars().count() <= 64);
    assert!(entry.chars().count() <= 60);
    assert!(entry.contains('…'));
    assert!(!entry.chars().any(char::is_control));
}

#[test]
fn formats_sessions_normalize_controls_and_align_entries() {
    let rendered = format_value(&json!({
        "sessions": ["a", "b"],
        "active": "a",
        "inventory": [
            {"id": "a", "title": "  First\u{0000} session\n", "state": "recorded", "catalog_compatible": true},
            {"id": "b", "title": "Second", "state": "interrupted", "catalog_compatible": null}
        ]
    }));
    let entries = rendered.lines().skip(2).collect::<Vec<_>>();

    assert_eq!(entries[0].chars().count(), entries[1].chars().count());
    assert!(entries.iter().all(|entry| entry.chars().count() <= 60));
    assert!(
        entries
            .iter()
            .all(|entry| !entry.chars().any(char::is_control))
    );
    assert!(entries[0].contains("First session"));
    assert!(!entries[0].contains("recorded"));
}

#[test]
fn formats_sessions_without_mutating_or_exposing_json_shape() {
    let value = json!({
        "sessions": ["0123456789abcdef"],
        "active": "0123456789abcdef",
        "inventory": [{"id": "0123456789abcdef", "title": "Build dashboard", "state": "recorded", "catalog_compatible": true}]
    });
    let original = value.clone();

    let rendered = format_value(&value);

    assert!(!rendered.contains("inventory:"));
    assert!(!rendered.contains("0123456789abcdef"));
    assert_eq!(value, original);
    assert_eq!(value["sessions"][0], "0123456789abcdef");
}

#[test]
fn resume_labels_match_compact_aligned_session_style() {
    let label = super::resume_session_label(&json!({
        "id": "0123456789abcdef0123456789abcdef",
        "title": "  Build\u{0000}   dashboard  ",
        "initial_prompt_preview": "Build dashboard with safe colors",
        "state": "interrupted",
        "catalog_compatible": false
    }));
    assert_eq!(label, "Build dashboard                  01234567 ×!");
    assert_eq!(label.chars().count(), 44);
    assert!(!label.contains("safe colors"));
}

#[test]
fn resume_labels_are_bounded_and_keep_status_fields() {
    let label = super::resume_session_label(&json!({
        "id": "0123456789abcdef0123456789abcdef",
        "title": "A title that is intentionally very long and should be clipped without wrapping",
        "state": "recorded",
        "catalog_compatible": true
    }));
    assert!(label.chars().count() <= 44);
    assert!(label.contains("01234567"));
    assert!(label.ends_with("✓ "));
    assert!(label.contains('…'));
}

#[test]
fn resume_labels_use_normalized_preview_as_title_when_title_is_absent() {
    let label = super::resume_session_label(&json!({
        "id": "0123456789abcdef0123456789abcdef",
        "title": null,
        "initial_prompt_preview": " Start   a new dashboard\n",
        "state": "recorded",
        "catalog_compatible": null
    }));
    assert!(label.starts_with("Start a new dashboard"));
    assert!(label.ends_with("01234567 ? "));
}

#[test]
fn formats_scalar_values_without_json_quotes() {
    assert_eq!(format_value(&json!("hello")), "hello");
    assert_eq!(format_value(&json!(42)), "42");
    assert_eq!(format_value(&json!(false)), "false");
    assert_eq!(format_value(&json!(null)), "null");
}

#[test]
fn formats_project_local_sdd_status_as_an_actionable_summary() {
    assert_eq!(
        format_value(&json!({"changes": [], "canonical_surface": false})),
        "SDD status\nSurface: project-local SDD (.agents/changes/)\nChanges: none\nNext: use /sdd init <change-id> [title] to create a change packet."
    );
}

#[test]
fn formats_canonical_sdd_status_without_exposing_internal_keys() {
    let rendered = format_value(&json!({
        "changes": ["add-search", "improve-ui"],
        "canonical_surface": true
    }));
    assert_eq!(
        rendered,
        "SDD status\nSurface: canonical SDD (specs/ or openspec/)\nChanges (2):\n  - add-search\n  - improve-ui\nNext: add or update a change in specs/ or openspec/."
    );
    assert!(!rendered.contains("canonical_surface"));
}

#[test]
fn formats_file_approval_as_bounded_human_change_summary() {
    let rendered = format_approval(&json!({
        "call": {"name": "write_file"},
        "details": {
            "path": "views/layouts/site-layout.dowe",
            "skill": "views",
            "reason": "Alinear la navegación",
            "before": "layout SiteLayout\n  main\n",
            "after": "layout SiteLayout\n  Scaffold\n    appBar\n      AppBar\n"
        }
    }));

    assert!(rendered.starts_with("File change requested"));
    assert!(rendered.contains("path: views/layouts/site-layout.dowe"));
    assert!(rendered.contains("before:"));
    assert!(rendered.contains("after:"));
    assert!(rendered.contains("Alinear la navegación"));
    assert!(!rendered.contains("\"call\""));
    assert!(!rendered.contains('{'));
    assert!(rendered.lines().count() <= 28);
}

#[test]
fn approval_preview_does_not_dump_large_file_bodies_or_controls() {
    let rendered = format_approval(&json!({
        "call": {"name": "write_file"},
        "details": {
            "path": "src/main.rs",
            "before": "before\u{1b}[2J\n".to_string() + &"x\n".repeat(100),
            "after": "after\n"
        }
    }));

    assert!(
        !rendered
            .chars()
            .any(|character| character.is_control() && !matches!(character, '\n' | '\t'))
    );
    assert!(rendered.contains("preview truncated"));
    assert!(rendered.len() < 3000);
}

#[test]
fn formats_a_file_approval_batch_as_one_bounded_review() {
    let rendered = format_approval_batch(&[
        json!({
            "call": {"name": "write_file"},
            "details": {
                "path": "views/pages/home.dowe",
                "reason": "Create the page",
                "before": "old\n",
                "after": "new\n"
            }
        }),
        json!({
            "call": {"name": "edit_file"},
            "details": {
                "path": "views/pages/home.dowe",
                "reason": "Refine the CTA",
                "before": "new\n",
                "after": "final\n"
            }
        }),
    ]);

    assert!(rendered.starts_with("File changes requested (2)"));
    assert!(rendered.contains("Operation 1:"));
    assert!(rendered.contains("Operation 2:"));
    assert_eq!(rendered.matches("views/pages/home.dowe").count(), 2);
    assert!(rendered.contains("These operations are single-use"));
    assert!(rendered.lines().count() <= 97);
}
