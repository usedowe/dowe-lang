use crate::menus;
use crate::usage::USAGE;
use dowe_codegraph::{BuildOptions, CheckOptions, CheckReport, write_codegraph_baseline};
use std::env;
use std::fs;

pub(crate) async fn run_codegraph_command(
    args: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    let interactive_args;
    let args = if args.is_empty() && menus::is_interactive_terminal() {
        let Some(command) = menus::prompt_codegraph_command()? else {
            return Ok(());
        };
        interactive_args = vec![command];
        interactive_args.as_slice()
    } else {
        args
    };
    let root = env::current_dir()?;

    match args.first().map(String::as_str) {
        Some("source") if args.len() == 1 => {
            let graph = dowe_codegraph::clean::build_clean_graph(
                &root,
                BuildOptions {
                    mode: Some(dowe_codegraph::CodeGraphMode::Project),
                },
            )?;
            let output_root = root
                .parent()
                .filter(|parent| {
                    parent.join("AGENTS.md").is_file() && parent.join("agents/README.md").is_file()
                })
                .unwrap_or(&root);
            let path = output_root.join("codegraph/source.json");
            if let Some(parent) = path.parent() {
                fs::create_dir_all(parent)?;
            }
            fs::write(&path, serde_json::to_vec_pretty(&graph)?)?;
            println!("written {}", path.display());
            Ok(())
        }
        Some("build") if args.len() == 1 => {
            let clean = dowe_codegraph::clean::build_clean_graph(&root, BuildOptions::default())?;
            let report = CheckReport::new();
            let written = dowe_codegraph::write_clean_codegraph_reports(&root, &clean, &report)?;
            println!("written {}", written.graph_path);
            Ok(())
        }
        Some("check") if args.len() == 1 => {
            let clean = dowe_codegraph::clean::build_clean_graph(&root, BuildOptions::default())?;
            let report =
                dowe_codegraph::clean::check_clean_codegraph(&root, CheckOptions::default())?;
            let written = dowe_codegraph::write_clean_codegraph_reports(&root, &clean, &report)?;
            print_report(&report);
            println!("written {}", written.report_path);
            if report.has_errors() {
                Err("codegraph check failed".into())
            } else {
                Ok(())
            }
        }
        Some("report") if args.len() == 1 => {
            let clean = dowe_codegraph::clean::build_clean_graph(&root, BuildOptions::default())?;
            let report =
                dowe_codegraph::clean::check_clean_codegraph(&root, CheckOptions::default())?;
            let written = dowe_codegraph::write_clean_codegraph_reports(&root, &clean, &report)?;
            let content = fs::read_to_string(root.join(&written.markdown_path))?;
            print!("{content}");
            Ok(())
        }
        Some("baseline") if args.len() == 1 => {
            let report =
                dowe_codegraph::clean::check_clean_codegraph(&root, CheckOptions::default())?;
            let path = write_codegraph_baseline(&root, &report)?;
            println!("written {path}");
            Ok(())
        }
        Some("explain") if args.len() == 2 => {
            let explanation =
                dowe_codegraph::explain_clean_node(&root, &args[1], BuildOptions::default())?;
            println!("id {}", explanation.node.id);
            println!("kind {}", explanation.node.kind);
            println!("namespace {:?}", explanation.node.namespace);
            println!("evidence {:?}", explanation.node.evidence);
            if let Some(path) = explanation.node.path {
                println!("path {path}");
            }
            if let (Some(start), Some(end)) =
                (explanation.node.start_line, explanation.node.end_line)
            {
                println!("lines {start}-{end}");
            }
            println!("incoming {}", explanation.incoming.len());
            println!("outgoing {}", explanation.outgoing.len());
            Ok(())
        }
        _ => Err(USAGE.into()),
    }
}

fn print_report(report: &CheckReport) {
    if report.diagnostics.is_empty() {
        println!("codegraph check passed");
        return;
    }

    for diagnostic in &report.diagnostics {
        eprintln!(
            "{:?} {} {}: {} ({})",
            diagnostic.severity,
            diagnostic.path,
            diagnostic.code,
            diagnostic.message,
            diagnostic.action
        );
    }
}
