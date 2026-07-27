use crate::usage::USAGE;
use dowe_agent::{init_external_agent_project, update_external_agent_project};
use dowe_agent_harness::InitReport;
use std::collections::BTreeSet;
use std::env;

pub(super) fn run_agent_init_command(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if !args.is_empty() {
        return Err(USAGE.into());
    }
    print_report(init_external_agent_project(env::current_dir()?)?, false);
    Ok(())
}

pub(super) fn run_agent_update_command(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if !args.is_empty() {
        return Err(USAGE.into());
    }
    print_report(update_external_agent_project(env::current_dir()?)?, true);
    Ok(())
}

fn print_report(report: InitReport, updating: bool) {
    let skill_names = report
        .created
        .iter()
        .chain(&report.preserved)
        .filter_map(|file| managed_skill_name(&file.path))
        .collect::<BTreeSet<_>>();
    let created_label = if updating { "updated" } else { "created" };
    for file in report
        .created
        .iter()
        .filter(|file| managed_skill_name(&file.path).is_none())
    {
        println!("{created_label} {}", file.path);
    }
    for file in report
        .preserved
        .iter()
        .filter(|file| managed_skill_name(&file.path).is_none())
    {
        println!("preserved {}", file.path);
    }
    for file in report.blocked {
        println!("blocked {}", file.path);
    }
    if !skill_names.is_empty() {
        let action = if updating {
            "updated"
        } else if report
            .created
            .iter()
            .any(|file| managed_skill_name(&file.path).is_some())
        {
            "installed"
        } else {
            "preserved"
        };
        println!(
            "{action} {} Dowe skills under .agents/skills",
            skill_names.len()
        );
    }
}

fn managed_skill_name(path: &str) -> Option<&str> {
    path.strip_prefix(".agents/skills/")?.split('/').next()
}
