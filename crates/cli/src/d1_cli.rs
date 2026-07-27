use crate::menus;
use dowe_runtime::generate_solar_icon_catalog_d1_migrations;
use std::env;

const D1_USAGE: &str =
    "Usage: dowe d1 migrations icon-catalog --output <project-relative-directory>";

pub(crate) fn run_d1_command(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let interactive_args;
    let args = if args.is_empty() {
        if !menus::is_interactive_terminal() {
            return Err(D1_USAGE.into());
        }
        interactive_args = vec![
            "migrations".to_string(),
            "icon-catalog".to_string(),
            "--output".to_string(),
            menus::prompt_d1_migrations_output()?,
        ];
        &interactive_args
    } else {
        args
    };

    match args {
        [migrations, catalog, output, directory]
            if migrations == "migrations" && catalog == "icon-catalog" && output == "--output" =>
        {
            generate_icon_catalog_migrations(directory)
        }
        _ => Err(D1_USAGE.into()),
    }
}

fn generate_icon_catalog_migrations(output: &str) -> Result<(), Box<dyn std::error::Error>> {
    let report = generate_solar_icon_catalog_d1_migrations(env::current_dir()?, output)?;
    println!(
        "Generated {} D1 migrations with {} Solar icon variants in {}",
        report.migrations,
        report.icon_variants,
        report.output.display()
    );
    Ok(())
}
