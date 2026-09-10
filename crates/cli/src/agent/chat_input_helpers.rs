fn is_exit_command(command: &str) -> bool {
    command == "/exit"
}

fn read_agent_prompt(
    footer: &super::footer::Footer<'_>,
    json_output: bool,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    if !json_output && is_interactive_terminal() {
        return Ok(super::prompt::read_interactive_prompt(footer)?);
    }
    if !json_output {
        print!("\n> ");
        io::stdout().flush()?;
    }
    let mut prompt = String::new();
    if io::stdin().read_line(&mut prompt)? == 0 {
        return Ok(None);
    }
    Ok(Some(prompt.trim_end_matches(['\r', '\n']).to_string()))
}
