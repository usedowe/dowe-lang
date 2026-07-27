use crate::menus;
use crate::usage::USAGE;
use dowe_agent::project_context;
use std::env;

pub(super) fn run_agent_context_command(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let interactive_args;
    let args = if args.is_empty() && menus::is_interactive_terminal() {
        interactive_args = ["project".to_string()];
        interactive_args.as_slice()
    } else {
        args
    };

    let json = match args {
        [project] if project == "project" => false,
        [project, json] if project == "project" && json == "--json" => true,
        _ => return Err(USAGE.into()),
    };
    let context = project_context(env::current_dir()?)?;
    if json {
        println!("{}", serde_json::to_string(&context)?);
    } else {
        println!("root {}", context.root);
        println!("mode {}", context.mode);
        println!("dowe {}", context.dowe_version);
        println!("source files {}", context.source_file_count);
        println!("skills {}", context.skills.len());
        println!("harness {}", context.harness.mode);
        println!("codegraph nodes {}", context.codegraph.node_count);
    }
    Ok(())
}
