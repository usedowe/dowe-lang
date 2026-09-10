use std::{
    collections::VecDeque,
    future::Future,
    io::Write,
    sync::{Arc, Mutex, MutexGuard},
    time::{Duration, Instant},
};

use crossterm::{
    cursor,
    event::{self, Event, KeyCode, KeyEventKind, KeyModifiers},
    execute, terminal,
};
use dowe_agent::{AgentError, AgentResult};

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn busy_input_edits_and_enqueues_fifo_with_validation() {
        let activity = Activity::new(true).unwrap();
        let mut state = activity.state();
        for ch in "first".chars() {
            state.input.handle(KeyCode::Char(ch));
        }
        state.input.handle(KeyCode::Left);
        state.input.handle(KeyCode::Backspace);
        assert_eq!(state.input.value(), "firt");
        state.submit_input();
        for index in 0..7 {
            state.pending.push_back(format!("p{index}"));
        }
        assert_eq!(
            state.pending.iter().collect::<Vec<_>>(),
            vec!["firt", "p0", "p1", "p2", "p3", "p4", "p5", "p6"]
        );
        assert!(state.input.value().is_empty());
        state.input.handle(KeyCode::Char('x'));
        state.submit_input();
        assert_eq!(state.input_status.as_deref(), Some("queue full (limit 8)"));
    }

    #[test]
    fn collapsed_activity_hides_tool_details_but_expanded_activity_keeps_them() {
        let activity = Activity::new(true).unwrap();
        activity.push(["shell [one] · completed".into(), "  result: output".into()]);
        let mut state = activity.state();
        let collapsed = state.frame((120, 20), false).join("\n");
        assert!(collapsed.contains("shell [one] · completed"));
        assert!(!collapsed.contains("result: output"));
        state.expanded = true;
        let expanded = state.frame((120, 20), false).join("\n");
        assert!(expanded.contains("shell [one] · completed"));
        assert!(expanded.contains("result: output"));
    }

    #[test]
    fn busy_input_keeps_pending_entries_when_turn_is_canceled() {
        let activity = Activity::new(true).unwrap();
        {
            let mut state = activity.state();
            state.pending.extend(["first".into(), "second".into()]);
        }
        assert_eq!(activity.take_pending(), ["first", "second"]);
    }

    #[test]
    fn inline_frames_bound_cells_and_keep_busy_chrome_below_activity() {
        let activity = Activity::new(true).unwrap();
        activity.footer(["fixture directory".into(), "fixture model".into()]);
        activity.push(["界".repeat(200)]);
        let mut state = activity.state();
        for cols in [0, 1, 2, 8, 80, 120] {
            for rows in [0, 1, 2, 3, 5, 12, 60] {
                for expanded in [false, true] {
                    state.expanded = expanded;
                    let frame = state.frame((cols, rows), true);
                    assert!(frame.len() <= rows.saturating_sub(2) as usize);
                    assert!(frame.iter().all(|line| {
                        dialoguer::console::measure_text_width(line)
                            <= cols.saturating_sub(1) as usize
                    }));
                }
            }
        }
        state.expanded = false;
        assert!(
            state
                .frame((120, 60), true)
                .iter()
                .any(|line| line.contains("Working…"))
        );
        assert!(
            state
                .frame((120, 60), true)
                .iter()
                .any(|line| line.contains("Agents"))
        );
        state.expanded = true;
        assert!(
            state
                .frame((120, 60), true)
                .iter()
                .any(|line| line.contains("Working…"))
        );
        assert!(!state.frame((120, 60), false).join("\n").contains("Working"));
    }

    #[test]
    fn workspace_state_uses_monotonic_elapsed_time_and_native_transitions() {
        let activity = Activity::new(true).unwrap();
        activity.workspace_event(
            &serde_json::json!({"event":"request_prepared","role":"execute","model":"safe/model"}),
        );
        let state = activity.state();
        let first = state.workspace_lines().join(" ");
        assert!(first.contains("in_progress") && first.contains("safe/model"));
        drop(state);
        std::thread::sleep(Duration::from_millis(2));
        let state = activity.state();
        assert!(state.workspace_lines().join(" ").contains("0s"));
        drop(state);
        activity.workspace_event(&serde_json::json!({"event":"approval_required"}));
        assert!(
            activity
                .state()
                .workspace_lines()
                .join(" ")
                .contains("awaiting_approval")
        );
    }

    #[test]
    fn workspace_cards_are_bounded_on_tiny_terminals() {
        let activity = Activity::new(true).unwrap();
        activity.workspace_event(
            &serde_json::json!({"event":"request_prepared","role":"execute","model":"m"}),
        );
        let state = activity.state();
        for size in [(0, 0), (4, 4), (20, 8)] {
            let frame = state.frame(size, true);
            assert!(frame.len() <= size.1.saturating_sub(2) as usize);
            assert!(
                frame
                    .iter()
                    .all(|line| dialoguer::console::measure_text_width(line)
                        <= size.0.saturating_sub(1) as usize)
            );
        }
    }

    #[test]
    fn collapsed_cards_leave_tool_activity_visible() {
        let activity = Activity::new(true).unwrap();
        activity.workspace_event(
            &serde_json::json!({"event":"request_prepared","role":"execute","model":"m"}),
        );
        activity.push(["retained-pipe-marker".into()]);
        let state = activity.state();
        let frame = state.frame((120, 20), true);
        assert!(frame.iter().any(|line| line.contains("Agents")));
        assert!(frame.iter().any(|line| line.contains("Todos")));
        assert!(frame.iter().any(|line| line.contains("Activity collapsed")));
        assert!(
            frame
                .iter()
                .any(|line| line.contains("retained-pipe-marker"))
        );
        assert!(frame.iter().any(|line| line.contains("Ctrl+O")));
    }

    #[test]
    fn inline_ownership_is_invalidated_on_resize_and_snapshot_includes_last_push() {
        let activity = Activity::new(true).unwrap();
        let mut bytes = Vec::new();
        {
            let mut state = activity.state();
            state.paint(&mut bytes, (120, 60), true).unwrap();
            assert_eq!(state.owned, 11);
            bytes.clear();
            state.clear_owned(&mut bytes, (40, 8)).unwrap();
            assert_eq!(bytes, b"\r\n", "resize must not rewind or erase stale rows");
            assert_eq!(state.owned, 0);
        }
        activity.push(["last synchronous result".into()]);
        let mut state = activity.state();
        // A ready future can finish before the next timer repaint.
        assert!(state.snapshot_pending);
        bytes.clear();
        state.paint(&mut bytes, (120, 60), false).unwrap();
        let snapshot = String::from_utf8(bytes).unwrap();
        assert!(snapshot.contains("last synchronous result"));
        assert!(!snapshot.contains("Working"));
        assert_eq!(state.owned, 0);
    }

    #[test]
    fn stream_chunks_coalesce_without_control_injection_or_unbounded_lines() {
        let activity = Activity::new(true).unwrap();
        activity.stream("preview", "first");
        activity.stream("preview", " second\n");
        activity.stream("stdout", "\u{1b}[2Jpipe");
        {
            let state = activity.state();
            assert_eq!(state.lines.len(), 2);
            assert_eq!(state.lines[0], "preview: first second");
            assert!(!state.lines[1].chars().any(char::is_control));
        }
        activity.stream("stdout", &"x".repeat(100_000));
        let state = activity.state();
        assert_eq!(state.lines.len(), 2);
        assert!(state.lines[1].len() <= super::super::LINE_BYTES);
        assert!(state.omitted);
    }

    #[test]
    fn retention_is_bounded_and_nested_suspensions_relinquish_input() {
        let activity = Activity::new(true).unwrap();
        for _ in 0..200 {
            activity.push(["x".repeat(2000)]);
        }
        {
            let state = activity.state();
            assert_eq!(state.lines.len(), 128);
            assert!(state.omitted);
            assert!(
                state
                    .lines
                    .iter()
                    .all(|line| line.len() <= super::super::LINE_BYTES)
            );
        }
        let first = activity.suspend().unwrap();
        let second = activity.suspend().unwrap();
        assert_eq!(activity.state().suspended, 2);
        assert!(!activity.state().active);
        drop(second);
        assert_eq!(activity.state().suspended, 1);
        drop(first);
        assert_eq!(activity.state().suspended, 0);
        assert!(!activity.state().active);
    }
}

#[derive(Clone)]
pub(crate) struct Activity(Arc<Mutex<State>>);

struct State {
    enabled: bool,
    active: bool,
    suspended: usize,
    expanded: bool,
    offset: usize,
    older: bool,
    omitted: bool,
    dirty: bool,
    lines: VecDeque<String>,
    stream: String,
    stream_open: bool,
    error: Option<String>,
    owned: u16,
    anchor: Option<(u16, u16)>,
    snapshot_pending: bool,
    footer: [String; 2],
    agent_name: String,
    agent_role: String,
    agent_model: String,
    task_title: String,
    task_status: String,
    started_at: Option<Instant>,
    input: EditableLine,
    pending: VecDeque<String>,
    input_status: Option<String>,
    spinner: usize,
    animated_at: Instant,
}

#[derive(Default)]
struct EditableLine {
    text: Vec<char>,
    cursor: usize,
}

