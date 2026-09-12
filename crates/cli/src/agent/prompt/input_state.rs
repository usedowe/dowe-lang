#[derive(Default)]
struct Prompt {
    text: Vec<char>,
    cursor: usize,
    selected: usize,
    dismissed: bool,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct InputRow {
    text: String,
    start: usize,
    end: usize,
    width: usize,
}

#[derive(Clone, Debug, PartialEq, Eq)]
struct InputView {
    rows: Vec<String>,
    cursor_row: usize,
    cursor_column: usize,
}

impl Prompt {
    fn value(&self) -> String {
        self.text.iter().collect()
    }

    fn matches(&self) -> Vec<&'static str> {
        let value = self.value();
        if self.dismissed || !value.starts_with('/') {
            return Vec::new();
        }
        VISIBLE_COMMANDS
            .iter()
            .copied()
            .filter(|command| command.starts_with(&value))
            .collect()
    }

    fn handle(&mut self, key: Key, width: usize) -> Option<Option<String>> {
        let matches = self.matches();
        match key {
            Key::Enter => {
                return Some(Some(
                    matches
                        .get(self.selected)
                        .map_or_else(|| self.value(), |command| command.to_string()),
                ));
            }
            Key::CtrlC => return Some(None),
            Key::Char('\u{4}') if self.text.is_empty() => return Some(None),
            Key::Char('\n') => {
                self.insert_at_cursor('\n');
                self.reset();
            }
            Key::ArrowUp if !matches.is_empty() => {
                self.selected = (self.selected + matches.len() - 1) % matches.len();
            }
            Key::ArrowDown if !matches.is_empty() => {
                self.selected = (self.selected + 1) % matches.len();
            }
            Key::ArrowUp => self.move_vertical(width, -1),
            Key::ArrowDown => self.move_vertical(width, 1),
            Key::Escape => self.dismissed = true,
            Key::ArrowLeft => self.cursor = self.cursor.saturating_sub(1),
            Key::ArrowRight => self.cursor = (self.cursor + 1).min(self.text.len()),
            Key::Home => {
                let layout = self.layout(width);
                self.cursor = layout.rows[layout.cursor_row].start;
            }
            Key::End => {
                let layout = self.layout(width);
                self.cursor = layout.rows[layout.cursor_row].end;
            }
            Key::Backspace if self.cursor > 0 => {
                self.cursor -= 1;
                self.text.remove(self.cursor);
                self.reset();
            }
            Key::Del if self.cursor < self.text.len() => {
                self.text.remove(self.cursor);
                self.reset();
            }
            Key::Char(ch) if !ch.is_control() => {
                self.insert_at_cursor(ch);
                self.reset();
            }
            _ => {}
        }
        None
    }

    fn insert_text(&mut self, text: &str) {
        let mut changed = false;
        let mut carriage_return = false;
        for ch in text.chars() {
            match ch {
                '\r' => {
                    self.insert_at_cursor('\n');
                    carriage_return = true;
                    changed = true;
                }
                '\n' => {
                    if !carriage_return {
                        self.insert_at_cursor('\n');
                        changed = true;
                    }
                    carriage_return = false;
                }
                ch => {
                    carriage_return = false;
                    if !ch.is_control() {
                        self.insert_at_cursor(ch);
                        changed = true;
                    }
                }
            }
        }
        if changed {
            self.reset();
        }
    }

    fn insert_at_cursor(&mut self, ch: char) {
        self.text.insert(self.cursor, ch);
        self.cursor += 1;
    }

    fn reset(&mut self) {
        self.selected = 0;
        self.dismissed = false;
    }

    fn layout(&self, width: usize) -> InputLayout {
        let mut rows = Vec::new();
        let mut row_start = 0;
        let mut row_end = 0;
        let mut row_text = String::new();
        let mut row_width = 0;
        for (index, ch) in self.text.iter().copied().enumerate() {
            if ch == '\n' {
                push_input_row(
                    &mut rows,
                    row_start,
                    row_end,
                    std::mem::take(&mut row_text),
                    row_width,
                );
                row_start = index + 1;
                row_end = row_start;
                row_width = 0;
                continue;
            }
            let ch_width = measure_text_width(&ch.to_string());
            if width > 0 && !row_text.is_empty() && row_width + ch_width > width {
                push_input_row(
                    &mut rows,
                    row_start,
                    row_end,
                    std::mem::take(&mut row_text),
                    row_width,
                );
                row_start = index;
                row_width = 0;
            }
            if width > 0 && row_width + ch_width <= width {
                row_text.push(ch);
                row_width += ch_width;
            }
            row_end = index + 1;
        }
        push_input_row(&mut rows, row_start, row_end, row_text, row_width);

        if width > 0
            && self.cursor == self.text.len()
            && let Some(row_index) = rows
                .iter()
                .position(|row| row.start < row.end && row.end == self.cursor && row.width == width)
            && !rows
                .get(row_index + 1)
                .is_some_and(|row| row.start == self.cursor && row.end == self.cursor)
        {
            rows.insert(
                row_index + 1,
                InputRow {
                    text: String::new(),
                    start: self.cursor,
                    end: self.cursor,
                    width: 0,
                },
            );
        }

        let cursor_row = rows
            .iter()
            .enumerate()
            .rev()
            .find_map(|(index, row)| {
                (self.cursor >= row.start && self.cursor <= row.end).then_some(index)
            })
            .unwrap_or(0);
        let row = &rows[cursor_row];
        let cursor_column = self.text[row.start..self.cursor.min(row.end)]
            .iter()
            .filter(|ch| **ch != '\n')
            .map(|ch| measure_text_width(&ch.to_string()))
            .sum::<usize>();
        InputLayout {
            rows,
            cursor_row,
            cursor_column,
        }
    }

    fn input_view(&self, width: usize, max_rows: usize) -> InputView {
        let layout = self.layout(width);
        let max_rows = max_rows.max(1);
        let first = layout.cursor_row.saturating_sub(max_rows - 1);
        let last = (first + max_rows).min(layout.rows.len());
        InputView {
            rows: layout.rows[first..last]
                .iter()
                .map(|row| row.text.clone())
                .collect(),
            cursor_row: layout.cursor_row - first,
            cursor_column: layout.cursor_column.min(width.saturating_sub(1)),
        }
    }

    fn move_vertical(&mut self, width: usize, direction: isize) {
        let layout = self.layout(width);
        let target = if direction < 0 {
            layout.cursor_row.saturating_sub(1)
        } else {
            (layout.cursor_row + 1).min(layout.rows.len() - 1)
        };
        if target != layout.cursor_row {
            self.cursor = self.cursor_at_column(&layout.rows[target], layout.cursor_column);
        }
    }

    fn cursor_at_column(&self, row: &InputRow, column: usize) -> usize {
        let mut index = row.start;
        let mut used = 0;
        while index < row.end {
            let ch = self.text[index];
            if ch == '\n' {
                break;
            }
            let ch_width = measure_text_width(&ch.to_string());
            if used + ch_width > column {
                break;
            }
            used += ch_width;
            index += 1;
        }
        index
    }
}

struct InputLayout {
    rows: Vec<InputRow>,
    cursor_row: usize,
    cursor_column: usize,
}

fn push_input_row(rows: &mut Vec<InputRow>, start: usize, end: usize, text: String, width: usize) {
    rows.push(InputRow {
        text,
        start,
        end,
        width,
    });
}
