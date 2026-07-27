use crate::usage::USAGE;

pub(crate) fn run_version_command(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if !args.is_empty() {
        return Err(USAGE.into());
    }

    println!("dowe {}", version_text());
    Ok(())
}

fn version_text() -> &'static str {
    env!("CARGO_PKG_VERSION")
}

#[cfg(test)]
mod tests {
    use super::version_text;

    #[test]
    fn reports_the_package_version() {
        assert_eq!(version_text(), env!("CARGO_PKG_VERSION"));
    }
}
