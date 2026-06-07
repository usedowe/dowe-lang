use crate::menus;
use crate::usage::USAGE;
use dowe_agent_harness::{
    CheckReport, InitOptions, PlanOptions, check_harness, init_project_harness, plan_from_spec,
    read_status, validate_plan,
};
use std::env;
use std::path::PathBuf;

pub(crate) async fn run_agent_command(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    match args.first().map(String::as_str) {
        Some("harness") => run_agent_harness_command(&args[1..]).await,
        None if menus::is_interactive_terminal() => {
            let Some(command) = menus::prompt_agent_command()? else {
                return Ok(());
            };
            match command.as_str() {
                "harness" => run_agent_harness_command(&[]).await,
                _ => Err(USAGE.into()),
            }
        }
        _ => Err(USAGE.into()),
    }
}

async fn run_agent_harness_command(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let interactive_args;
    let args = if args.is_empty() && menus::is_interactive_terminal() {
        let Some(command) = menus::prompt_harness_command()? else {
            return Ok(());
        };
        interactive_args = vec![command];
        interactive_args.as_slice()
    } else {
        args
    };

    let root = env::current_dir()?;

    match args.first().map(String::as_str) {
        Some("init") if args.len() == 1 => {
            let report = init_project_harness(root, InitOptions::default())?;
            for file in report.created {
                println!("created {}", file.path);
            }
            for file in report.preserved {
                println!("preserved {}", file.path);
            }
            for file in report.blocked {
                println!("blocked {}", file.path);
            }
            Ok(())
        }
        Some("check") if args.len() == 1 => {
            let report = check_harness(root)?;
            print_check_report(&report);
            if report.has_errors() {
                Err("agent harness check failed".into())
            } else {
                Ok(())
            }
        }
        Some("status") if args.len() == 1 => {
            let status = read_status(root)?;
            println!("mode {:?}", status.mode);
            for plan in status.plans {
                println!(
                    "plan {} {:?} complete={}",
                    plan.plan_id, plan.state, plan.complete
                );
            }
            Ok(())
        }
        Some("plan") => {
            let spec = required_option(&args[1..], "--spec")?;
            let report = plan_from_spec(root, PathBuf::from(spec), PlanOptions::default())?;
            println!("plan {}", report.plan_id);
            println!("written {}", report.plan_path);
            println!("written {}", report.state_path);
            println!("complete {}", report.complete);
            Ok(())
        }
        Some("validate") => {
            let plan = required_option(&args[1..], "--plan")?;
            let report = validate_plan(root, &plan)?;
            println!("validation {}", report.plan_id);
            println!("success {}", report.success);
            println!("evidence {}", report.evidence_path);
            if report.success {
                Ok(())
            } else {
                Err("agent harness validation failed".into())
            }
        }
        _ => Err(USAGE.into()),
    }
}

fn required_option(args: &[String], name: &str) -> Result<String, Box<dyn std::error::Error>> {
    if args.len() == 2 && args[0] == name {
        Ok(args[1].clone())
    } else {
        Err(USAGE.into())
    }
}

fn print_check_report(report: &CheckReport) {
    if report.diagnostics.is_empty() {
        println!("agent harness check passed");
        return;
    }

    for diagnostic in &report.diagnostics {
        eprintln!(
            "{} {}: {} ({})",
            diagnostic.path, diagnostic.code, diagnostic.message, diagnostic.action
        );
    }
}
