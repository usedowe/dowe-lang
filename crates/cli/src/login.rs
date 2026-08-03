use dialoguer::{Password, theme::ColorfulTheme};

const LOGIN_USAGE: &str = "Usage: dowe login";

pub(crate) fn run_login_command(args: &[String]) -> Result<(), Box<dyn std::error::Error>> {
    if !args.is_empty() || !crate::menus::is_interactive_terminal() {
        return Err(LOGIN_USAGE.into());
    }
    let token = Password::with_theme(&ColorfulTheme::default())
        .with_prompt("Dowe Cloud API token")
        .interact()?;
    dowe_deploy::authenticate_cloud_session(&token)?;
    println!("Authenticated with Dowe Cloud");
    Ok(())
}
