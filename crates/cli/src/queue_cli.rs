use dowe_queue::{
    QueueServerConfig, create_account, init_namespace, list_namespaces, open_namespace,
    start_queue_server,
};
use serde_json::Value;
use std::env;

pub(crate) async fn run_queue_command(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    let root = env::current_dir()?;
    match args.first().map(String::as_str) {
        Some("start") => {
            let (host, port) = parse_start_options(&args[1..])?;
            let server = start_queue_server(QueueServerConfig { root, host, port }).await?;
            println!("Dowe Queue listening at ws://{}", server.addr);
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
            println!("queue account {} {}", created.name, created.account);
            if created.generated {
                println!("secret {}", created.secret);
            } else {
                println!("secret stored");
            }
        }
        Some("init") => {
            let name = required_exact(args, 1, "missing Queue name")?;
            init_namespace(&root, name)?;
            println!("initialized {name}");
        }
        Some("list") => {
            if args.len() != 1 {
                return Err(usage().into());
            }
            for name in list_namespaces(&root)? {
                println!("{name}");
            }
        }
        Some("inspect") => {
            let name = required_exact(args, 1, "missing Queue name")?;
            let inspection = open_namespace(&root, name)?.inspect()?;
            println!("queue {}", inspection.name);
            match inspection.queues {
                Some(queues) => {
                    for queue in queues {
                        println!("queue {}", queue.queue);
                        println!("ready {}", queue.ready);
                        println!("inFlight {}", queue.in_flight);
                        for binding in queue.bindings {
                            println!("binding {binding}");
                        }
                    }
                }
                None => println!("queues unknown"),
            }
        }
        Some("declare") => {
            let (name, queue) = pair(args, "missing Queue name", "missing Queue name")?;
            let report = open_namespace(&root, name)?.declare(queue)?;
            println!("declared {} {}", report.queue, known_bool(report.created));
        }
        Some("bind") => {
            let (name, queue, pattern) = triple(
                args,
                "missing Queue name",
                "missing Queue name",
                "missing Queue topic pattern",
            )?;
            let report = open_namespace(&root, name)?.bind(queue, pattern)?;
            println!(
                "bound {} {} {}",
                report.queue,
                report.pattern,
                known_bool(report.created)
            );
        }
        Some("publish") => {
            let (name, topic, value) = triple(
                args,
                "missing Queue name",
                "missing Queue topic",
                "missing Queue JSON value",
            )?;
            let value = serde_json::from_str::<Value>(value)?;
            let report = open_namespace(&root, name)?.publish(topic, value)?;
            println!("published {}", report.id);
            match report.destinations {
                Some(destinations) => {
                    for destination in destinations {
                        println!("destination {destination}");
                    }
                }
                None => println!("destinations unknown"),
            }
        }
        Some("purge") => {
            let (name, queue) = pair(args, "missing Queue name", "missing Queue name")?;
            let report = open_namespace(&root, name)?.purge(queue)?;
            println!("purged {} {}", report.queue, report.removed);
        }
        _ => return Err(usage().into()),
    }
    Ok(())
}

fn known_bool(value: Option<bool>) -> &'static str {
    match value {
        Some(true) => "true",
        Some(false) => "false",
        None => "unknown",
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct CreateAccountOptions {
    name: String,
    account: String,
    secret: Option<String>,
}

fn parse_start_options(args: &[String]) -> Result<(String, u16), Box<dyn std::error::Error>> {
    let mut host = "127.0.0.1".to_string();
    let mut port = 4150;
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
            value => return Err(format!("unknown queue start option `{value}`").into()),
        }
    }
    Ok((host, port))
}

fn parse_create_account_options(
    args: &[String],
) -> Result<CreateAccountOptions, Box<dyn std::error::Error>> {
    let name = required(args, 1, "missing Queue name")?.to_string();
    let account = required(args, 2, "missing Queue account")?.to_string();
    let mut secret = None;
    let mut index = 3;
    while index < args.len() {
        match args[index].as_str() {
            "--secret" => {
                secret = Some(args.get(index + 1).ok_or("missing --secret value")?.clone());
                index += 2;
            }
            value => return Err(format!("unknown queue create-account option `{value}`").into()),
        }
    }
    Ok(CreateAccountOptions {
        name,
        account,
        secret,
    })
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

fn pair<'a>(
    args: &'a [String],
    first_message: &str,
    second_message: &str,
) -> Result<(&'a str, &'a str), Box<dyn std::error::Error>> {
    if args.len() != 3 {
        return Err(usage().into());
    }
    Ok((
        required(args, 1, first_message)?,
        required(args, 2, second_message)?,
    ))
}

fn triple<'a>(
    args: &'a [String],
    first_message: &str,
    second_message: &str,
    third_message: &str,
) -> Result<(&'a str, &'a str, &'a str), Box<dyn std::error::Error>> {
    if args.len() != 4 {
        return Err(usage().into());
    }
    Ok((
        required(args, 1, first_message)?,
        required(args, 2, second_message)?,
        required(args, 3, third_message)?,
    ))
}

fn usage() -> &'static str {
    "Usage: dowe queue start [--host <host>] [--port <port>] | create-account <name> <account> [--secret <secret>] | init <name> | list | inspect <name> | declare <name> <queue> | bind <name> <queue> <topic-pattern> | publish <name> <topic> <json> | purge <name> <queue>"
}

#[cfg(test)]
mod tests {
    use super::{
        CreateAccountOptions, known_bool, parse_create_account_options, parse_start_options,
    };

    #[test]
    fn parses_queue_start_defaults_and_options() {
        assert_eq!(
            parse_start_options(&[]).expect("defaults"),
            ("127.0.0.1".to_string(), 4150)
        );
        let args = vec![
            "--host".to_string(),
            "0.0.0.0".to_string(),
            "--port".to_string(),
            "5150".to_string(),
        ];
        assert_eq!(
            parse_start_options(&args).expect("options"),
            ("0.0.0.0".to_string(), 5150)
        );
    }

    #[test]
    fn parses_queue_account_secret() {
        let args = vec![
            "create-account".to_string(),
            "orders".to_string(),
            "orders-api".to_string(),
            "--secret".to_string(),
            "secret".to_string(),
        ];
        assert_eq!(
            parse_create_account_options(&args).expect("options"),
            CreateAccountOptions {
                name: "orders".to_string(),
                account: "orders-api".to_string(),
                secret: Some("secret".to_string()),
            }
        );
    }

    #[test]
    fn renders_unknown_provider_facts_without_false_values() {
        assert_eq!(known_bool(None), "unknown");
        assert_eq!(known_bool(Some(false)), "false");
    }
}
