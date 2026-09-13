pub(crate) fn input_outline(width: usize, label: &str) -> [String; 2] {
    let label = truncate_str(label, width.saturating_sub(4), "");
    [
        format!(
            "╭─{label}{}╮",
            "─".repeat(width.saturating_sub(3 + measure_text_width(&label)))
        ),
        format!("╰{}╯", "─".repeat(width.saturating_sub(2))),
    ]
}

struct PromptFrame {
    line_widths: Vec<usize>,
    cursor_line: usize,
    cursor_column: usize,
}

impl PromptFrame {
    fn cursor_row_after_reflow(&self, columns: usize) -> usize {
        let columns = columns.max(1);
        let cursor_line = self.cursor_line.min(self.line_widths.len());
        self.line_widths[..cursor_line]
            .iter()
            .map(|width| reflow_rows(*width, columns))
            .sum::<usize>()
            .saturating_add(self.cursor_column / columns)
    }
}

fn reflow_rows(width: usize, columns: usize) -> usize {
    width.max(1).div_ceil(columns.max(1))
}

fn clear_resized_frame(term: &Term, frame: &PromptFrame, size: (u16, u16)) -> io::Result<()> {
    term.move_cursor_up(frame.cursor_row_after_reflow(usize::from(size.1)))?;
    term.write_str("\r")?;
    term.clear_to_end_of_screen()
}

fn clear_frame(term: &Term, frame: &PromptFrame) -> io::Result<()> {
    term.move_cursor_up(frame.cursor_line)?;
    term.write_str("\r")?;
    term.clear_to_end_of_screen()
}

fn anchor_frame(term: &Term, frame_rows: usize, size: (u16, u16), scroll: bool) -> io::Result<()> {
    term.move_cursor_down(usize::from(size.0))?;
    if scroll {
        for _ in 0..frame_rows {
            term.write_str("\r\n")?;
        }
    }
    term.move_cursor_up(frame_rows.saturating_sub(1))?;
    term.write_str("\r")
}

pub(super) fn read_interactive_prompt(
    footer: &super::footer::Footer<'_>,
    initial: Option<&str>,
) -> io::Result<Option<String>> {
    let _input_terminal = PromptTerminal::open()?;
    let term = Term::stdout();
    let mut prompt = Prompt::from_text(initial.unwrap_or_default());
    let mut anchor = term.size();
    let mut frame: Option<PromptFrame> = None;
    loop {
        let size = term.size();
        let width = usize::from(size.1).saturating_sub(1);
        let outlined = width >= 16 && size.0 >= 8;
        let below = usize::from(size.0.saturating_sub(2)).min(if outlined { 3 } else { 2 });
        let matches = prompt.matches();
        let suggestion_rows = matches
            .len()
            .min(usize::from(size.0).saturating_sub(if outlined { 6 } else { 4 }));
        let first = if suggestion_rows == 0 {
            0
        } else {
            prompt.selected.saturating_sub(suggestion_rows - 1)
        };
        let mut frame_lines = Vec::new();
        for (index, command) in matches.iter().enumerate().skip(first).take(suggestion_rows) {
            let line = if index == prompt.selected {
                format!("{} {}", style("❯").green(), style(command).cyan())
            } else {
                format!("  {}", style(command).dim())
            };
            frame_lines.push(truncate_str(&line, width, "").into_owned());
        }
        let prefix = if outlined { "│ > " } else { "> " };
        let prefix_width = measure_text_width(prefix);
        let inner = width.saturating_sub(prefix_width + usize::from(outlined) * 2);
        let border_rows = usize::from(outlined) * 2;
        let input_limit = usize::from(size.0)
            .saturating_sub(below + suggestion_rows + border_rows)
            .min(MAX_INPUT_ROWS)
            .max(1);
        let input = prompt.input_view(inner, input_limit);
        if outlined {
            let [top, _] = input_outline(width, "");
            frame_lines.push(style(&top).cyan().to_string());
        }
        let input_start = frame_lines.len();
        for (index, line) in input.rows.iter().enumerate() {
            frame_lines.push(styled_input_line(line, index, outlined, width, inner));
        }
        let [_, bottom] = input_outline(width, "");
        let footer_lines = footer.lines(width);
        let tail: Vec<_> = if outlined {
            vec![
                style(&bottom).cyan().to_string(),
                footer_lines[0].clone(),
                footer_lines[1].clone(),
            ]
        } else {
            footer_lines.to_vec()
        };
        frame_lines.extend(tail.iter().take(below).cloned());
        let first_frame = frame.is_none();
        if let Some(previous) = frame.as_ref() {
            if size == anchor {
                clear_frame(&term, previous)?;
            } else {
                clear_resized_frame(&term, previous, size)?;
            }
        } else {
            term.write_str("\r")?;
        }
        anchor = size;
        anchor_frame(&term, frame_lines.len(), size, first_frame)?;
        for (index, line) in frame_lines.iter().enumerate() {
            if index > 0 {
                term.write_str("\r\n")?;
            }
            term.write_str(line)?;
        }
        let cursor_line = input_start + input.cursor_row;
        let cursor_column = prefix_width + input.cursor_column;
        term.move_cursor_up(frame_lines.len().saturating_sub(cursor_line + 1))?;
        term.write_str("\r")?;
        term.move_cursor_right(cursor_column)?;
        frame = Some(PromptFrame {
            line_widths: frame_lines
                .iter()
                .map(|line| measure_text_width(line))
                .collect(),
            cursor_line,
            cursor_column,
        });
        let event = read_prompt_event()?;
        let result = match event {
            PromptEvent::Key(key) => prompt.handle(key, inner),
            PromptEvent::Paste(text) => {
                prompt.insert_text(&text);
                None
            }
            PromptEvent::Resize => None,
        };
        if let Some(result) = result {
            let final_size = term.size();
            if final_size == anchor {
                if let Some(previous) = frame.as_ref() {
                    clear_frame(&term, previous)?;
                }
            } else if let Some(previous) = frame.as_ref() {
                clear_resized_frame(&term, previous, final_size)?;
            } else {
                term.write_str("\r\n")?;
            }
            write_prompt_line(
                &term,
                &format!(
                    "{} {}",
                    style("dowe >").cyan().bold(),
                    result.as_deref().unwrap_or_default()
                ),
            )?;
            return Ok(result);
        }
    }
}

fn styled_input_line(
    text: &str,
    index: usize,
    outlined: bool,
    width: usize,
    inner: usize,
) -> String {
    if outlined {
        let prefix = if index == 0 {
            format!("{} {} ", style("│").cyan(), style(">").cyan().bold())
        } else {
            format!("{}   ", style("│").cyan())
        };
        format!(
            "{prefix}{text}{}{}",
            " ".repeat(inner.saturating_sub(measure_text_width(text)) + 1),
            style("│").cyan()
        )
    } else {
        let prefix = if index == 0 {
            format!("{} ", style(">").cyan().bold())
        } else {
            "  ".into()
        };
        truncate_str(&format!("{prefix}{text}"), width, "").into_owned()
    }
}
