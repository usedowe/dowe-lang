use dowe_store::{
    StoreServerConfig, create_user, init_database, list_databases, open_database, run_bench,
    start_store_server,
};
use std::env;

pub(crate) async fn run_store_command(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let root = env::current_dir()?;
    match args.first().map(String::as_str) {
        Some("start") => {
            let options = parse_start_options(&args[1..])?;
            let server = start_store_server(StoreServerConfig {
                root,
                host: options.host,
                port: options.port,
            })
            .await?;
            println!("Store server listening at http://{}", server.addr);
            server.wait().await?;
        }
        Some("create-user") | Some("createUser") => {
            let options = parse_create_user_options(args)?;
            let created = create_user(
                &root,
                &options.database,
                &options.user,
                options.credential.as_deref(),
            )?;
            println!("store user {} {}", created.database, created.user);
            if created.generated {
                println!("token {}", created.credential);
            } else {
                println!("credential stored");
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
    "Usage: dowe store start [--host <host>] [--port <port>] | create-user <database> <user> [--token <token> | --password <password>] | init <database> | list | inspect <database> | query <database> <sql> | index <database> <table> <field> | compact <database> | bench"
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct StoreStartOptions {
    host: String,
    port: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CreateUserOptions {
    database: String,
    user: String,
    credential: Option<String>,
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
            value => return Err(format!("unknown store start option `{value}`").into()),
        }
    }
    Ok(options)
}

fn parse_create_user_options(
    args: &[String],
) -> Result<CreateUserOptions, Box<dyn std::error::Error>> {
    let database = required(args, 1, "missing database name")?.to_string();
    let user = required(args, 2, "missing user name")?.to_string();
    let mut token = None::<String>;
    let mut password = None::<String>;
    let mut index = 3usize;
    while index < args.len() {
        match args[index].as_str() {
            "--token" => {
                let Some(value) = args.get(index + 1) else {
                    return Err("missing --token value".into());
                };
                token = Some(value.clone());
                index += 2;
            }
            "--password" => {
                let Some(value) = args.get(index + 1) else {
                    return Err("missing --password value".into());
                };
                password = Some(value.clone());
                index += 2;
            }
            value => return Err(format!("unknown store create-user option `{value}`").into()),
        }
    }
    if token.is_some() && password.is_some() {
        return Err("store create-user accepts either --token or --password, not both".into());
    }
    Ok(CreateUserOptions {
        database,
        user,
        credential: token.or(password),
    })
}

#[cfg(test)]
mod tests {
    use super::{
        CreateUserOptions, StoreStartOptions, parse_create_user_options, parse_start_options,
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
    fn parses_store_create_user_credentials() {
        let args = vec![
            "create-user".to_string(),
            "clinic".to_string(),
            "clinic-api".to_string(),
            "--token".to_string(),
            "secret".to_string(),
        ];
        assert_eq!(
            parse_create_user_options(&args).expect("options"),
            CreateUserOptions {
                database: "clinic".to_string(),
                user: "clinic-api".to_string(),
                credential: Some("secret".to_string()),
            }
        );
    }

    #[test]
    fn rejects_two_store_create_user_credentials() {
        let args = vec![
            "create-user".to_string(),
            "clinic".to_string(),
            "clinic-api".to_string(),
            "--token".to_string(),
            "secret".to_string(),
            "--password".to_string(),
            "other".to_string(),
        ];
        let error = parse_create_user_options(&args).expect_err("error");
        assert!(error.to_string().contains("either --token or --password"));
    }
}
