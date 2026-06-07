use crate::menus;
use crate::usage::USAGE;
use dowe_codegraph::{
    BuildOptions, CheckOptions, CheckReport, build_codegraph, check_codegraph, explain_node,
    write_codegraph_baseline, write_codegraph_reports,
};
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
        Some("build") if args.len() == 1 => {
            let graph = build_codegraph(&root, BuildOptions::default())?;
            let report = CheckReport::new();
            let written = write_codegraph_reports(&root, &graph, &report)?;
            println!("written {}", written.graph_path);
            Ok(())
        }
        Some("check") if args.len() == 1 => {
            let graph = build_codegraph(&root, BuildOptions::default())?;
            let report = check_codegraph(&root, CheckOptions::default())?;
            let written = write_codegraph_reports(&root, &graph, &report)?;
            print_report(&report);
            println!("written {}", written.report_path);
            if report.has_errors() {
                Err("codegraph check failed".into())
            } else {
                Ok(())
            }
        }
        Some("report") if args.len() == 1 => {
            let graph = build_codegraph(&root, BuildOptions::default())?;
            let report = check_codegraph(&root, CheckOptions::default())?;
            let written = write_codegraph_reports(&root, &graph, &report)?;
            let content = fs::read_to_string(root.join(&written.markdown_path))?;
            print!("{content}");
            Ok(())
        }
        Some("baseline") if args.len() == 1 => {
            let report = check_codegraph(&root, CheckOptions::default())?;
            let path = write_codegraph_baseline(&root, &report)?;
            println!("written {path}");
            Ok(())
        }
        Some("explain") if args.len() == 2 => {
            let explanation = explain_node(&root, &args[1], BuildOptions::default())?;
            println!("id {}", explanation.node.id);
            println!("kind {:?}", explanation.node.kind);
            if let Some(path) = explanation.node.path {
                println!("path {path}");
            }
            if let Some(owner) = explanation.node.owner {
                println!("owner {owner}");
            }
            if let Some(metrics) = explanation.node.metrics {
                println!("lines {}", metrics.total_lines);
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
