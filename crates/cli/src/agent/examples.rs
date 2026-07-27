use crate::menus;
use crate::usage::USAGE;
use dowe_agent::search_public_examples;

pub(super) fn run_agent_examples_command(
    args: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    let interactive_args;
    let args = if args.is_empty() && menus::is_interactive_terminal() {
        let Some(query) = menus::prompt_agent_example_query()? else {
            return Ok(());
        };
        interactive_args = vec!["search".to_string(), query];
        interactive_args.as_slice()
    } else {
        args
    };

    if args.first().map(String::as_str) != Some("search") {
        return Err(USAGE.into());
    }
    let json = args.iter().skip(1).any(|arg| arg == "--json");
    if args
        .iter()
        .skip(1)
        .any(|arg| arg.starts_with("--") && arg != "--json")
    {
        return Err(USAGE.into());
    }
    let query = args
        .iter()
        .skip(1)
        .filter(|arg| arg.as_str() != "--json")
        .map(String::as_str)
        .collect::<Vec<_>>()
        .join(" ");
    let search = search_public_examples(&query, 5)?;
    if json {
        println!("{}", serde_json::to_string(&search)?);
    } else {
        for (index, result) in search.results.into_iter().enumerate() {
            if index > 0 {
                println!();
            }
            println!("{}", result.title);
            println!("skill {} score {}", result.skill, result.score);
            println!("source {}", result.source_path);
            println!();
            println!("{}", result.content);
        }
    }
    Ok(())
}
