use dowe_kv::{
    KvServerConfig, create_user, init_database, list_databases, open_database, start_kv_server,
};
use serde_json::Value;
use std::env;

pub(crate) async fn run_kv_command(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let root = env::current_dir()?;
    match args.first().map(String::as_str) {
        Some("start") => {
            let options = parse_start_options(&args[1..])?;
            let server = start_kv_server(KvServerConfig {
                root,
                host: options.host,
                port: options.port,
            })
            .await?;
            println!("KV server listening at http://{}", server.addr);
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
            println!("kv user {} {}", created.database, created.user);
            if created.generated {
                println!("token {}", created.credential);
            } else {
                println!("credential stored");
            }
        }
        Some("init") => {
            let database = required(args, 1, "missing database name")?;
            init_database(&root, database)?;
            println!("initialized {database}");
        }
        Some("list") => {
            for database in list_databases(&root)? {
                println!("{database}");
            }
        }
        Some("inspect") => {
            let options = parse_database_options(args, 1)?;
            let db = open_database(&root, &options.database, options.persist)?;
            let inspection = db.inspect()?;
            println!("database {}", inspection.name);
            println!("persistent {}", inspection.persistent);
            println!("memoryKeys {}", inspection.memory_keys);
            println!("persistedKeys {}", inspection.persisted_keys);
            for key in inspection.keys {
                println!("key {key}");
            }
        }
        Some("set") => {
            let options = parse_set_options(args)?;
            let value = serde_json::from_str::<Value>(&options.value)?;
            let db = open_database(&root, &options.database, options.persist)?;
            let report = db.set(&options.key, value)?;
            println!("set {}", report.key);
        }
        Some("get") => {
            let options = parse_key_options(args, 1)?;
            let db = open_database(&root, &options.database, options.persist)?;
            let value = db.get(&options.key)?;
            if options.required && value.is_none() {
                return Err("KV key not found".into());
            }
            println!(
                "{}",
                serde_json::to_string_pretty(&value.unwrap_or(Value::Null))?
            );
        }
        Some("delete") => {
            let options = parse_key_options(args, 1)?;
            let db = open_database(&root, &options.database, options.persist)?;
            println!("deleted {}", db.delete(&options.key)?);
        }
        Some("keys") => {
            let options = parse_keys_options(args)?;
            let db = open_database(&root, &options.database, options.persist)?;
            for key in db.keys(options.prefix.as_deref())? {
                println!("{key}");
            }
        }
        Some("clear") => {
            let options = parse_database_options(args, 1)?;
            let db = open_database(&root, &options.database, options.persist)?;
            println!("cleared {}", db.clear()?);
        }
        _ => return Err(kv_usage().into()),
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

fn kv_usage() -> &'static str {
    "Usage: dowe kv start [--host <host>] [--port <port>] | create-user <database> <user> [--token <token> | --password <password>] | init <database> | list | inspect <database> [--persist] | set <database> <key> <json> [--persist] | get <database> <key> [--persist] [--required] | delete <database> <key> [--persist] | keys <database> [prefix] [--persist] | clear <database> [--persist]"
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct KvStartOptions {
    host: String,
    port: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CreateUserOptions {
    database: String,
    user: String,
    credential: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct DatabaseOptions {
    database: String,
    persist: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct KeyOptions {
    database: String,
    key: String,
    persist: bool,
    required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SetOptions {
    database: String,
    key: String,
    value: String,
    persist: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct KeysOptions {
    database: String,
    prefix: Option<String>,
    persist: bool,
}

fn parse_start_options(args: &[String]) -> Result<KvStartOptions, Box<dyn std::error::Error>> {
    let mut options = KvStartOptions {
        host: "127.0.0.1".to_string(),
        port: 4148,
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
            value => return Err(format!("unknown kv start option `{value}`").into()),
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
            value => return Err(format!("unknown kv create-user option `{value}`").into()),
        }
    }
    if token.is_some() && password.is_some() {
        return Err("kv create-user accepts either --token or --password, not both".into());
    }
    Ok(CreateUserOptions {
        database,
        user,
        credential: token.or(password),
    })
}

fn parse_database_options(
    args: &[String],
    database_index: usize,
) -> Result<DatabaseOptions, Box<dyn std::error::Error>> {
    let database = required(args, database_index, "missing database name")?.to_string();
    let persist = parse_flags(args, database_index + 1)?.persist;
    Ok(DatabaseOptions { database, persist })
}

fn parse_key_options(
    args: &[String],
    database_index: usize,
) -> Result<KeyOptions, Box<dyn std::error::Error>> {
    let database = required(args, database_index, "missing database name")?.to_string();
    let key = required(args, database_index + 1, "missing key")?.to_string();
    let flags = parse_flags(args, database_index + 2)?;
    Ok(KeyOptions {
        database,
        key,
        persist: flags.persist,
        required: flags.required,
    })
}

fn parse_set_options(args: &[String]) -> Result<SetOptions, Box<dyn std::error::Error>> {
    let database = required(args, 1, "missing database name")?.to_string();
    let key = required(args, 2, "missing key")?.to_string();
    let value = required(args, 3, "missing JSON value")?.to_string();
    let flags = parse_flags(args, 4)?;
    Ok(SetOptions {
        database,
        key,
        value,
        persist: flags.persist,
    })
}

fn parse_keys_options(args: &[String]) -> Result<KeysOptions, Box<dyn std::error::Error>> {
    let database = required(args, 1, "missing database name")?.to_string();
    let mut prefix = None::<String>;
    let mut persist = false;
    let mut index = 2usize;
    while index < args.len() {
        match args[index].as_str() {
            "--persist" => {
                persist = true;
                index += 1;
            }
            value if value.starts_with("--") => {
                return Err(format!("unknown kv keys option `{value}`").into());
            }
            value if prefix.is_none() => {
                prefix = Some(value.to_string());
                index += 1;
            }
            value => return Err(format!("unexpected kv keys argument `{value}`").into()),
        }
    }
    Ok(KeysOptions {
        database,
        prefix,
        persist,
    })
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Flags {
    persist: bool,
    required: bool,
}

fn parse_flags(args: &[String], start: usize) -> Result<Flags, Box<dyn std::error::Error>> {
    let mut flags = Flags {
        persist: false,
        required: false,
    };
    let mut index = start;
    while index < args.len() {
        match args[index].as_str() {
            "--persist" => flags.persist = true,
            "--required" => flags.required = true,
            value => return Err(format!("unknown kv option `{value}`").into()),
        }
        index += 1;
    }
    Ok(flags)
}

#[cfg(test)]
mod tests {
    use super::{
        CreateUserOptions, KvStartOptions, SetOptions, parse_create_user_options,
        parse_set_options, parse_start_options,
    };

    #[test]
    fn parses_kv_start_defaults_and_bind_options() {
        assert_eq!(
            parse_start_options(&[]).expect("defaults"),
            KvStartOptions {
                host: "127.0.0.1".to_string(),
                port: 4148,
            }
        );
        let args = vec![
            "--host".to_string(),
            "0.0.0.0".to_string(),
            "--port".to_string(),
            "5152".to_string(),
        ];
        assert_eq!(
            parse_start_options(&args).expect("options"),
            KvStartOptions {
                host: "0.0.0.0".to_string(),
                port: 5152,
            }
        );
    }

    #[test]
    fn parses_kv_create_user_credentials() {
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
    fn rejects_two_kv_create_user_credentials() {
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

    #[test]
    fn parses_kv_set_persistence() {
        let args = vec![
            "set".to_string(),
            "clinic".to_string(),
            "greeting".to_string(),
            "\"hello\"".to_string(),
            "--persist".to_string(),
        ];
        assert_eq!(
            parse_set_options(&args).expect("set"),
            SetOptions {
                database: "clinic".to_string(),
                key: "greeting".to_string(),
                value: "\"hello\"".to_string(),
                persist: true,
            }
        );
    }
}
