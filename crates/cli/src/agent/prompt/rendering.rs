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
pub(super) fn read_interactive_prompt(
    footer: &super::footer::Footer<'_>,
) -> io::Result<Option<String>> {
    let _input_terminal = PromptTerminal::open()?;
    let term = Term::stdout();
    let mut prompt = Prompt::default();
    let mut above_rows = 0;
    let mut input_height: usize = 1;
    let mut input_cursor_row = 0;
    let mut footer_rows = 0;
    let mut anchor = term.size();
    write_prompt_line(&term, "")?;
    loop {
        let size = term.size();
        if size == anchor {
            clear_footer(
                &term,
                footer_rows,
                input_height.saturating_sub(input_cursor_row + 1),
            )?;
            term.clear_line()?;
            term.clear_last_lines(input_height.saturating_sub(1))?;
            term.clear_last_lines(above_rows)?;
        } else {
            term.write_str("\r\n")?;
            anchor = size;
        }
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
        for (index, command) in matches.iter().enumerate().skip(first).take(suggestion_rows) {
            let line = if index == prompt.selected {
                format!("{} {}", style("❯").green(), style(command).cyan())
            } else {
                format!("  {}", style(command).dim())
            };
            write_prompt_line(&term, &truncate_str(&line, width, ""))?;
        }
        let above = suggestion_rows + usize::from(outlined);
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
            write_prompt_line(&term, &style(&top).cyan().to_string())?;
        }
        for (index, line) in input.rows.iter().enumerate() {
            if index > 0 {
                term.write_str("\r\n")?;
            }
            term.write_str(&styled_input_line(line, index, outlined, width, inner))?;
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
        for line in tail.iter().take(below) {
            term.write_str("\r\n")?;
            term.write_str(line)?;
        }
        let move_up = below + input.rows.len().saturating_sub(input.cursor_row + 1);
        term.move_cursor_up(move_up)?;
        term.write_str("\r")?;
        term.move_cursor_right(prefix_width + input.cursor_column)?;
        above_rows = above;
        input_height = input.rows.len();
        input_cursor_row = input.cursor_row;
        footer_rows = below;
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
            if term.size() == anchor {
                clear_footer(
                    &term,
                    footer_rows,
                    input_height.saturating_sub(input_cursor_row + 1),
                )?;
                term.clear_line()?;
                term.clear_last_lines(input_height.saturating_sub(1))?;
                term.clear_last_lines(above_rows)?;
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

fn clear_footer(term: &Term, rows: usize, cursor_offset: usize) -> io::Result<()> {
    term.move_cursor_down(cursor_offset)?;
    for _ in 0..rows {
        term.move_cursor_down(1)?;
        term.clear_line()?;
    }
    term.move_cursor_up(rows)
}
