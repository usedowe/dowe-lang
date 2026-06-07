use dowe_store::{init_database, list_databases, open_database, run_bench};
use std::env;

pub(crate) fn run_store_command(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let root = env::current_dir()?;
    match args.first().map(String::as_str) {
        Some("init") => {
            let database = required(args, 1, "missing database name")?;
            let metadata = init_database(&root, database)?;
            println!("initialized {}", metadata.name);
            println!("databaseId {}", metadata.database_id);
        }
        Some("list") => {
            for database in list_databases(&root)? {
                println!("{} {}", database.name, database.database_id);
            }
        }
        Some("inspect") => {
            let database = required(args, 1, "missing database name")?;
            let db = open_database(&root, database)?;
            let inspection = db.inspect()?;
            println!("database {}", inspection.name);
            println!("databaseId {}", inspection.database_id);
            println!("formatVersion {}", inspection.format_version);
            for table in inspection.tables {
                println!("table {} records {}", table.name, table.records);
                for index in table.indexes {
                    println!("index {}.{}", table.name, index);
                }
            }
        }
        Some("query") => {
            let database = required(args, 1, "missing database name")?;
            let sql = if args.len() > 2 {
                args[2..].join(" ")
            } else {
                return Err("missing query".into());
            };
            let db = open_database(&root, database)?;
            let value = db.query_json(&sql)?;
            println!("{}", serde_json::to_string_pretty(&value)?);
        }
        Some("index") => {
            let database = required(args, 1, "missing database name")?;
            let table = required(args, 2, "missing table name")?;
            let field = required(args, 3, "missing field name")?;
            let db = open_database(&root, database)?;
            let index = db.create_index(table, field)?;
            println!("index {}.{}", index.table, index.field);
        }
        Some("compact") => {
            let database = required(args, 1, "missing database name")?;
            let db = open_database(&root, database)?;
            let report = db.compact()?;
            println!(
                "compacted {} tables {} records {}",
                report.database, report.tables, report.records
            );
        }
        Some("bench") => {
            let report = run_bench(&root)?;
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
        _ => return Err(store_usage().into()),
    }
    Ok(())
}

fn required<'a>(
    args: &'a [String],
    index: usize,
    message: &str,
) -> Result<&'a str, Box<dyn std::error::Error>> {
    args.get(index)
        .map(String::as_str)
        .ok_or_else(|| message.into())
}

fn store_usage() -> &'static str {
    "Usage: dowe store init <database> | list | inspect <database> | query <database> <sql> | index <database> <table> <field> | compact <database> | bench"
}
