use dowe_cache::{
    CacheServerConfig, create_account, init_database, list_databases, open_database,
    start_cache_server,
};
use serde_json::Value;
use std::env;

pub(crate) async fn run_cache_command(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let root = env::current_dir()?;
    match args.first().map(String::as_str) {
        Some("start") => {
            let options = parse_start_options(&args[1..])?;
            let server = start_cache_server(CacheServerConfig {
                root,
                host: options.host,
                port: options.port,
            })
            .await?;
            println!("Dowe Cache listening at ws://{}", server.addr);
            server.wait().await?;
        }
        Some("create-account") | Some("createAccount") => {
            let options = parse_create_account_options(args)?;
            let created = create_account(
                &root,
                &options.name,
                &options.account,
                options.secret.as_deref(),
            )?;
            println!("cache account {} {}", created.name, created.account);
            if created.generated {
                println!("secret {}", created.secret);
            } else {
                println!("secret stored");
            }
        }
        Some("init") => {
            let name = required(args, 1, "missing Cache name")?;
            init_database(&root, name)?;
            println!("initialized {name}");
        }
        Some("list") => {
            for name in list_databases(&root)? {
                println!("{name}");
            }
        }
        Some("inspect") => {
            let name = required_exact(args, 1, "missing Cache name")?;
            let cache = open_database(&root, name, true)?;
            let inspection = cache.inspect()?;
            println!("cache {}", inspection.name);
            println!("memoryKeys {}", inspection.memory_keys);
            println!("persistedKeys {}", inspection.persisted_keys);
            for key in inspection.keys {
                println!("key {key}");
            }
        }
        Some("set") => {
            let options = parse_set_options(args)?;
            let value = serde_json::from_str::<Value>(&options.value)?;
            let cache = open_database(&root, &options.name, true)?;
            let report = cache.set(&options.key, value)?;
            println!("set {}", report.key);
        }
        Some("get") => {
            let options = parse_key_options(args, true)?;
            let cache = open_database(&root, &options.name, true)?;
            let value = cache.get(&options.key)?;
            if options.required && value.is_none() {
                return Err("Cache key not found".into());
            }
            println!(
                "{}",
                serde_json::to_string_pretty(&value.unwrap_or(Value::Null))?
            );
        }
        Some("delete") => {
            let options = parse_key_options(args, false)?;
            let cache = open_database(&root, &options.name, true)?;
            println!("deleted {}", cache.delete(&options.key)?);
        }
        Some("keys") => {
            let options = parse_keys_options(args)?;
            let cache = open_database(&root, &options.name, true)?;
            for key in cache.keys(options.prefix.as_deref())? {
                println!("{key}");
            }
        }
        Some("clear") => {
            let name = required_exact(args, 1, "missing Cache name")?;
            let cache = open_database(&root, name, true)?;
            println!("cleared {}", cache.clear()?);
        }
        _ => return Err(cache_usage().into()),
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

fn required_exact<'a>(
    args: &'a [String],
    index: usize,
    message: &str,
) -> Result<&'a str, Box<dyn std::error::Error>> {
    let value = required(args, index, message)?;
    if args.len() != index + 1 {
        return Err(cache_usage().into());
    }
    Ok(value)
}

fn cache_usage() -> &'static str {
    "Usage: dowe cache start [--host <host>] [--port <port>] | create-account <name> <account> [--secret <secret>] | init <name> | list | inspect <name> | set <name> <key> <json> | get <name> <key> [--required] | delete <name> <key> | keys <name> [prefix] | clear <name>"
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CacheStartOptions {
    host: String,
    port: u16,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CreateAccountOptions {
    name: String,
    account: String,
    secret: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct KeyOptions {
    name: String,
    key: String,
    required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct SetOptions {
    name: String,
    key: String,
    value: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct KeysOptions {
    name: String,
    prefix: Option<String>,
}

fn parse_start_options(args: &[String]) -> Result<CacheStartOptions, Box<dyn std::error::Error>> {
    let mut options = CacheStartOptions {
        host: "127.0.0.1".to_string(),
        port: 4148,
    };
    let mut index = 0usize;
    while index < args.len() {
        match args[index].as_str() {
            "--host" => {
                options.host = args.get(index + 1).ok_or("missing --host value")?.clone();
                index += 2;
            }
            "--port" => {
                options.port = args.get(index + 1).ok_or("missing --port value")?.parse()?;
                index += 2;
            }
            value => return Err(format!("unknown cache start option `{value}`").into()),
        }
    }
    Ok(options)
}

fn parse_create_account_options(
    args: &[String],
) -> Result<CreateAccountOptions, Box<dyn std::error::Error>> {
    let name = required(args, 1, "missing Cache name")?.to_string();
    let account = required(args, 2, "missing Cache account")?.to_string();
    let mut secret = None;
    let mut index = 3usize;
    while index < args.len() {
        match args[index].as_str() {
            "--secret" => {
                secret = Some(args.get(index + 1).ok_or("missing --secret value")?.clone());
                index += 2;
            }
            value => return Err(format!("unknown cache create-account option `{value}`").into()),
        }
    }
    Ok(CreateAccountOptions {
        name,
        account,
        secret,
    })
}

fn parse_key_options(
    args: &[String],
    allow_required: bool,
) -> Result<KeyOptions, Box<dyn std::error::Error>> {
    let name = required(args, 1, "missing Cache name")?.to_string();
    let key = required(args, 2, "missing Cache key")?.to_string();
    let mut required = false;
    for value in &args[3..] {
        if allow_required && value == "--required" {
            required = true;
        } else {
            return Err(format!("unknown cache option `{value}`").into());
        }
    }
    Ok(KeyOptions {
        name,
        key,
        required,
    })
}

fn parse_set_options(args: &[String]) -> Result<SetOptions, Box<dyn std::error::Error>> {
    let options = SetOptions {
        name: required(args, 1, "missing Cache name")?.to_string(),
        key: required(args, 2, "missing Cache key")?.to_string(),
        value: required(args, 3, "missing JSON value")?.to_string(),
    };
    if args.len() != 4 {
        return Err(cache_usage().into());
    }
    Ok(options)
}

fn parse_keys_options(args: &[String]) -> Result<KeysOptions, Box<dyn std::error::Error>> {
    let name = required(args, 1, "missing Cache name")?.to_string();
    if args.len() > 3 {
        return Err(cache_usage().into());
    }
    Ok(KeysOptions {
        name,
        prefix: args.get(2).cloned(),
    })
}

#[cfg(test)]
mod tests {
    use super::{
        CacheStartOptions, CreateAccountOptions, SetOptions, parse_create_account_options,
        parse_set_options, parse_start_options,
    };

    #[test]
    fn parses_cache_start_defaults_and_bind_options() {
        assert_eq!(
            parse_start_options(&[]).expect("defaults"),
            CacheStartOptions {
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
            CacheStartOptions {
                host: "0.0.0.0".to_string(),
                port: 5152,
            }
        );
    }

    #[test]
    fn parses_cache_account_secret() {
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
                name: "clinic".to_string(),
                account: "clinic-api".to_string(),
                secret: Some("secret".to_string()),
            }
        );
    }

    #[test]
    fn cache_set_is_persistent_without_a_flag() {
        let args = vec![
            "set".to_string(),
            "clinic".to_string(),
            "greeting".to_string(),
            "\"hello\"".to_string(),
        ];
        assert_eq!(
            parse_set_options(&args).expect("set"),
            SetOptions {
                name: "clinic".to_string(),
                key: "greeting".to_string(),
                value: "\"hello\"".to_string(),
            }
        );
    }
}
