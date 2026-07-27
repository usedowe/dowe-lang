use crate::usage::USAGE;
use dowe_compiler::{TestReport, TestStatus, run_project_tests};
use std::env;
use std::path::PathBuf;

pub(crate) fn run_test_command(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let mut json = false;
    let mut selectors = Vec::new();
    for argument in args {
        if argument == "--json" {
            json = true;
        } else if argument.starts_with("--") {
            return Err(USAGE.into());
        } else {
            selectors.push(PathBuf::from(argument));
        }
    }
    let report = run_project_tests(&env::current_dir()?, &selectors)?;
    if json {
        println!("{}", serde_json::to_string(&report)?);
    } else {
        print_report(&report);
    }
    if report.has_failures() {
        Err(format!("{} test(s) failed", report.failed).into())
    } else {
        Ok(())
    }
}

fn print_report(report: &TestReport) {
    for case in &report.cases {
        match case.status {
            TestStatus::Passed => println!(
                "PASS {}:{} {} ({} assertions)",
                case.path, case.line, case.name, case.assertions
            ),
            TestStatus::Failed => println!(
                "FAIL {}:{} {} {}",
                case.path,
                case.line,
                case.name,
                case.message.as_deref().unwrap_or("assertion failed")
            ),
        }
    }
    println!(
        "test result: {} passed; {} failed; {} discovered",
        report.passed, report.failed, report.discovered
    );
}

#[cfg(test)]
mod tests {
    use super::run_test_command;

    #[test]
    fn rejects_unknown_test_flags() {
        let error = run_test_command(&["--watch".to_string()]).expect_err("unknown flag");

        assert!(error.to_string().contains("Usage: dowe"));
    }
}
