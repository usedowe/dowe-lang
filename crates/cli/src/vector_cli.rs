use dowe_vector::{
    VectorServerConfig, create_account, init_database, list_databases, open_database,
    start_vector_server,
};
use serde_json::Value;
use std::env;

pub(crate) async fn run_vector_command(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let root = env::current_dir()?;
    match args.first().map(String::as_str) {
        Some("start") => {
            let (host, port) = parse_start_options(&args[1..])?;
            let server = start_vector_server(VectorServerConfig { root, host, port }).await?;
            println!("Dowe Vector listening at ws://{}", server.addr);
            server.wait().await?;
        }
        Some("create-account") | Some("createAccount") => {
            let name = required(args, 1, "missing Vector name")?;
            let account = required(args, 2, "missing Vector account")?;
            let secret = optional_named(args, 3, "--secret")?;
            let created = create_account(&root, name, account, secret.as_deref())?;
            println!("vector account {} {}", created.name, created.account);
            if created.generated {
                println!("secret {}", created.secret);
            } else {
                println!("secret stored");
            }
        }
        Some("init") => {
            let name = required_exact(args, 1, "missing Vector name")?;
            init_database(&root, name)?;
            println!("initialized {name}");
        }
        Some("list") => {
            if args.len() != 1 {
                return Err(usage().into());
            }
            for name in list_databases(&root)? {
                println!("{name}");
            }
        }
        Some("inspect") => {
            let name = required_exact(args, 1, "missing Vector name")?;
            let inspection = open_database(&root, name, true)?.inspect()?;
            println!("vector {}", inspection.name);
            println!(
                "dimensions {}",
                inspection
                    .dimensions
                    .map(|value| value.to_string())
                    .unwrap_or_else(|| "unset".to_string())
            );
            println!("embeddings {}", inspection.embeddings);
        }
        Some("upsert") => {
            if !(4..=5).contains(&args.len()) {
                return Err(usage().into());
            }
            let name = required(args, 1, "missing Vector name")?;
            let id = required(args, 2, "missing embedding ID")?;
            let vector = parse_vector(required(args, 3, "missing vector JSON")?)?;
            let metadata = args
                .get(4)
                .map(|value| serde_json::from_str::<Value>(value))
                .transpose()?
                .unwrap_or_else(|| Value::Object(Default::default()));
            let report = open_database(&root, name, true)?.upsert(id, vector, metadata)?;
            println!("{}", serde_json::to_string_pretty(&report)?);
        }
        Some("search") => {
            let options = parse_search_options(args)?;
            let matches = open_database(&root, &options.name, true)?.search(
                &options.vector,
                options.limit,
                options.min_score,
                options.filter.as_ref(),
            )?;
            println!("{}", serde_json::to_string_pretty(&matches)?);
        }
        Some("read") => {
            if args.len() != 3 {
                return Err(usage().into());
            }
            let database = open_database(&root, &args[1], true)?;
            println!(
                "{}",
                serde_json::to_string_pretty(
                    &database
                        .read(&args[2])?
                        .map(serde_json::to_value)
                        .transpose()?
                        .unwrap_or(Value::Null)
                )?
            );
        }
        Some("delete") => {
            if args.len() != 3 {
                return Err(usage().into());
            }
            println!(
                "deleted {}",
                open_database(&root, &args[1], true)?.delete(&args[2])?
            );
        }
        Some("entries") => {
            if args.len() != 2 {
                return Err(usage().into());
            }
            let entries = open_database(&root, &args[1], true)?.list(1000, None)?;
            println!("{}", serde_json::to_string_pretty(&entries)?);
        }
        _ => return Err(usage().into()),
    }
    Ok(())
}

struct SearchOptions {
    name: String,
    vector: Vec<f32>,
    limit: usize,
    min_score: f32,
    filter: Option<Value>,
}

fn parse_search_options(args: &[String]) -> Result<SearchOptions, Box<dyn std::error::Error>> {
    let name = required(args, 1, "missing Vector name")?.to_string();
    let vector = parse_vector(required(args, 2, "missing vector JSON")?)?;
    let mut options = SearchOptions {
        name,
        vector,
        limit: 10,
        min_score: -1.0,
        filter: None,
    };
    let mut index = 3;
    while index < args.len() {
        match args[index].as_str() {
            "--limit" => {
                options.limit = args
                    .get(index + 1)
                    .ok_or("missing --limit value")?
                    .parse()?;
                index += 2;
            }
            "--min-score" => {
                options.min_score = args
                    .get(index + 1)
                    .ok_or("missing --min-score value")?
                    .parse()?;
                index += 2;
            }
            "--where" => {
                options.filter = Some(serde_json::from_str(
                    args.get(index + 1).ok_or("missing --where value")?,
                )?);
                index += 2;
            }
            value => return Err(format!("unknown vector search option `{value}`").into()),
        }
    }
    Ok(options)
}

fn parse_vector(value: &str) -> Result<Vec<f32>, Box<dyn std::error::Error>> {
    let value = serde_json::from_str::<Value>(value)?;
    let values = value.as_array().ok_or("vector JSON must be an array")?;
    values
        .iter()
        .map(|value| {
            value
                .as_f64()
                .map(|value| value as f32)
                .ok_or_else(|| "vector dimensions must be numbers".into())
        })
        .collect()
}

fn parse_start_options(args: &[String]) -> Result<(String, u16), Box<dyn std::error::Error>> {
    let mut host = "127.0.0.1".to_string();
    let mut port = 4149;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--host" => {
                host = args.get(index + 1).ok_or("missing --host value")?.clone();
                index += 2;
            }
            "--port" => {
                port = args.get(index + 1).ok_or("missing --port value")?.parse()?;
                index += 2;
            }
            value => return Err(format!("unknown vector start option `{value}`").into()),
        }
    }
    Ok((host, port))
}

fn optional_named(
    args: &[String],
    index: usize,
    name: &str,
) -> Result<Option<String>, Box<dyn std::error::Error>> {
    if args.len() == index {
        return Ok(None);
    }
    if args.get(index).map(String::as_str) != Some(name) || args.len() != index + 2 {
        return Err(usage().into());
    }
    Ok(args.get(index + 1).cloned())
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
        return Err(usage().into());
    }
    Ok(value)
}

fn usage() -> &'static str {
    "Usage: dowe vector start [--host <host>] [--port <port>] | create-account <name> <account> [--secret <secret>] | init <name> | list | inspect <name> | upsert <name> <id> <vector-json> [metadata-json] | search <name> <vector-json> [--limit <n>] [--min-score <score>] [--where <json>] | read <name> <id> | delete <name> <id> | entries <name>"
}

#[cfg(test)]
mod tests {
    use super::{parse_search_options, parse_start_options, parse_vector};

    #[test]
    fn parses_vector_start_defaults() {
        assert_eq!(
            parse_start_options(&[]).expect("defaults"),
            ("127.0.0.1".to_string(), 4149)
        );
    }

    #[test]
    fn parses_vector_json_and_search_options() {
        assert_eq!(parse_vector("[1,0.5]").expect("vector"), vec![1.0, 0.5]);
        let options = parse_search_options(&[
            "search".to_string(),
            "articles".to_string(),
            "[1,0]".to_string(),
            "--limit".to_string(),
            "5".to_string(),
        ])
        .expect("options");
        assert_eq!(options.limit, 5);
    }
}
