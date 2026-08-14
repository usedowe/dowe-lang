use crate::menus;
use dowe_compiler::{compile_dev, compile_dev_with_seeders, generate_database_migrations};
use dowe_database::{
    DatabaseServiceConfig, create_account, init_database, list_databases, open_database, run_bench,
    start_database_service,
};
use dowe_runtime::seed_local_databases;
use std::env;

pub(crate) async fn run_database_command(
    args: &[String],
) -> Result<(), Box<dyn std::error::Error>> {
    let interactive_args;
    let args = if args.is_empty() {
        if !menus::is_interactive_terminal() {
            return Err(
                "dowe database requires a subcommand when no interactive terminal is available"
                    .into(),
            );
        }
        let Some(command) = menus::prompt_database_command()? else {
            return Ok(());
        };
        let Some(command_args) = menus::prompt_database_command_args(&command)? else {
            return Err(database_usage().into());
        };
        interactive_args = command_args;
        interactive_args.as_slice()
    } else {
        args
    };
    let root = env::current_dir()?;
    match args.first().map(String::as_str) {
        Some("start") => {
            let options = parse_start_options(&args[1..])?;
            let server = start_database_service(DatabaseServiceConfig {
                root,
                host: options.host,
                port: options.port,
            })
            .await?;
            println!("Dowe Database listening at ws://{}", server.addr);
            server.wait().await?;
        }
        Some("create-account") => {
            let options = parse_create_account_options(args)?;
            let created = create_account(
                &root,
                &options.database,
                &options.account,
                options.secret.as_deref(),
            )?;
            println!("database account {} {}", created.database, created.account);
            if created.generated {
                println!("secret {}", created.secret);
            } else {
                println!("secret stored");
            }
        }
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
        Some("seeders") => {
            if args.len() > 1 {
                return Err("dowe database seeders does not accept arguments".into());
            }
            let project = compile_dev_with_seeders(&root)?;
            let databases = project.databases.len();
            seed_local_databases(project).await?;
            println!("applied local seeders for {databases} database(s)");
        }
        Some("migrate") => {
            if args.len() > 1 {
                return Err("dowe database migrate does not accept arguments".into());
            }
            let project = compile_dev(&root)?;
            let report = generate_database_migrations(&project)?;
            println!(
                "database migrations created {} unchanged {} dynamic {}",
                report.created, report.unchanged, report.dynamic
            );
        }
        _ => return Err(database_usage().into()),
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

fn database_usage() -> &'static str {
    "Usage: dowe database start [--host <host>] [--port <port>] | create-account <database> <account> [--secret <secret>] | init <database> | list | inspect <database> | query <database> <sql> | index <database> <table> <field> | compact <database> | bench | migrate | seeders"
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct StoreStartOptions {
    host: String,
    port: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CreateAccountOptions {
    database: String,
    account: String,
    secret: Option<String>,
}

fn parse_start_options(args: &[String]) -> Result<StoreStartOptions, Box<dyn std::error::Error>> {
    let mut options = StoreStartOptions {
        host: "127.0.0.1".to_string(),
        port: 4147,
    };
    let mut index = 0usize;
    while index < args.len() {
        match args[index].as_str() {
            "--host" => {
                let Some(host) = args.get(index + 1) else {
                    return Err("missing --host value".into());
                };
                options.host = host.clone();
                index += 2;
            }
            "--port" => {
                let Some(port) = args.get(index + 1) else {
                    return Err("missing --port value".into());
                };
                options.port = port.parse()?;
                index += 2;
            }
            value => return Err(format!("unknown database start option `{value}`").into()),
        }
    }
    Ok(options)
}

fn parse_create_account_options(
    args: &[String],
) -> Result<CreateAccountOptions, Box<dyn std::error::Error>> {
    let database = required(args, 1, "missing database name")?.to_string();
    let account = required(args, 2, "missing account name")?.to_string();
    let mut secret = None::<String>;
    let mut index = 3usize;
    while index < args.len() {
        match args[index].as_str() {
            "--secret" => {
                let Some(value) = args.get(index + 1) else {
                    return Err("missing --secret value".into());
                };
                secret = Some(value.clone());
                index += 2;
            }
            value => {
                return Err(format!("unknown database create-account option `{value}`").into());
            }
        }
    }
    Ok(CreateAccountOptions {
        database,
        account,
        secret,
    })
}

#[cfg(test)]
mod tests {
    use super::{
        CreateAccountOptions, StoreStartOptions, parse_create_account_options, parse_start_options,
    };

    #[test]
    fn parses_store_start_defaults_and_bind_options() {
        assert_eq!(
            parse_start_options(&[]).expect("defaults"),
            StoreStartOptions {
                host: "127.0.0.1".to_string(),
                port: 4147,
            }
        );
        let args = vec![
            "--host".to_string(),
            "0.0.0.0".to_string(),
            "--port".to_string(),
            "5151".to_string(),
        ];
        assert_eq!(
            parse_start_options(&args).expect("options"),
            StoreStartOptions {
                host: "0.0.0.0".to_string(),
                port: 5151,
            }
        );
    }

    #[test]
    fn parses_database_create_account_secret() {
        let args = vec![
            "create-account".to_string(),
            "clinic".to_string(),
            "clinic-api".to_string(),
            "--secret".to_string(),
            "secret".to_string(),
        ];
        assert_eq!(
            parse_create_account_options(&args).expect("options"),
            CreateAccountOptions {
                database: "clinic".to_string(),
                account: "clinic-api".to_string(),
                secret: Some("secret".to_string()),
            }
        );
    }

    #[test]
    fn database_usage_includes_manual_seeders() {
        assert!(super::database_usage().contains("seeders"));
        assert!(super::database_usage().contains("migrate"));
        for command in crate::menus::database_commands() {
            assert!(super::database_usage().contains(command));
        }
    }

    #[tokio::test]
    async fn database_without_subcommand_requires_interactive_terminal() {
        let error = super::run_database_command(&[])
            .await
            .expect_err("non-interactive database menu");
        assert_eq!(
            error.to_string(),
            "dowe database requires a subcommand when no interactive terminal is available"
        );
    }

    #[tokio::test]
    async fn database_seeders_rejects_extra_arguments() {
        let args = vec!["seeders".to_string(), "--remote".to_string()];
        let error = super::run_database_command(&args)
            .await
            .expect_err("unsupported arguments");
        assert_eq!(
            error.to_string(),
            "dowe database seeders does not accept arguments"
        );
    }

    #[tokio::test]
    async fn database_migrate_rejects_extra_arguments() {
        let args = vec!["migrate".to_string(), "--remote".to_string()];
        let error = super::run_database_command(&args)
            .await
            .expect_err("unsupported arguments");
        assert_eq!(
            error.to_string(),
            "dowe database migrate does not accept arguments"
        );
    }
}
