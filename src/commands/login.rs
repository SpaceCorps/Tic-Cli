//! `tic login`. Authenticates with TIC via API token, storing the key in the OS keystore.

use std::io::{BufRead, IsTerminal, Write};

use crate::cli::LoginArgs;
use crate::client::Client;
use crate::commands::print;
use crate::config::{self, AccountConfig};
use crate::error::{Error, Result};
use crate::obj;
use crate::secrets;

const TIC_DASHBOARD_URL: &str = "https://tic.io";

pub fn run(args: LoginArgs) -> Result<()> {
    let LoginArgs { name, api_key, api_key_stdin, no_browser, force, no_verify } = args;

    let name = name.trim().to_string();
    if name.is_empty() {
        return Err(Error::invalid("An account name is required."));
    }

    let store = secrets::store()?;
    let config = config::load()?;

    let existing = config.find(&name).map(|(k, _)| k.clone());
    if let Some(existing) = &existing
        && !force
    {
        return Err(Error::invalid(format!("An account named '{existing}' already exists."))
            .fix(format!("Use --force to replace its key: tic login {existing} --force")));
    }
    let name = existing.clone().unwrap_or(name);

    let key = if api_key_stdin {
        read_stdin_key()?
    } else if let Some(k) = api_key.map(|k| k.trim().to_string()).filter(|k| !k.is_empty()) {
        k
    } else {
        prompt_login_key(&name, no_browser)?
    };

    let mut identity = String::new();
    if !no_verify {
        let client = Client::new(&key);
        // Probe TIC API to verify key
        match client.get("me") {
            Ok(me) => {
                if let Some(email) = me.get("email").and_then(|v| v.as_str()) {
                    identity = email.to_string();
                } else if let Some(user_name) = me.get("name").and_then(|v| v.as_str()) {
                    identity = user_name.to_string();
                }
            }
            Err(e) if e.code == crate::error::ErrorCode::NotFound => {
                // If /me is not implemented, verify key against company search
                client.search("companies", "*", "registrationNumber", None, None, Some(1))?;
            }
            Err(e) => return Err(e),
        }
    }

    {
        let _lock = config::lock()?;
        store.set(&secrets::account_key(&name), &key)?;

        let mut config = config::load()?;
        config.accounts.insert(name.clone(), AccountConfig { identity: identity.clone(), added_at: config::now_utc() });
        config::save(&config)?;
    }

    if std::io::stderr().is_terminal() {
        if !identity.is_empty() {
            eprintln!("Successfully logged in as {identity} to account '{name}'.");
        } else {
            eprintln!("Successfully logged in to account '{name}'.");
        }
    }

    print(obj! {
        "status" => "logged_in",
        "name" => name,
        "identity" => identity,
        "verified" => !no_verify,
        "secretStore" => store.name(),
        "configDir" => config::config_dir().display().to_string(),
        "nextStep" => format!("tic company search Volvo -a {name}"),
    });

    Ok(())
}

fn read_stdin_key() -> Result<String> {
    let mut key = String::new();
    std::io::stdin().lock().read_line(&mut key).map_err(|e| Error::invalid(format!("Could not read stdin: {e}")))?;
    let key = key.trim().to_string();
    if key.is_empty() {
        return Err(Error::invalid("--api-key-stdin was given but stdin was empty."));
    }
    Ok(key)
}

fn prompt_login_key(name: &str, no_browser: bool) -> Result<String> {
    if !std::io::stdin().is_terminal() {
        return Err(Error::invalid("No API key given and no terminal to prompt on.")
            .fix(format!("pbpaste | tic login {name} --api-key-stdin")));
    }

    eprintln!("To log in, copy or create an API token from TIC:");
    eprintln!("  {TIC_DASHBOARD_URL}\n");

    if !no_browser {
        eprintln!("Opening {TIC_DASHBOARD_URL} in your browser...");
        open_browser(TIC_DASHBOARD_URL);
    }

    let _ = std::io::stderr().flush();

    loop {
        let key = rpassword::prompt_password(format!("Paste your TIC API token for '{name}': "))
            .map_err(|e| Error::other("Could not read the API key.").detail(e.to_string()))?;
        let key = key.trim().to_string();
        if !key.is_empty() {
            return Ok(key);
        }
        eprintln!("Token cannot be empty. Paste your API token from {TIC_DASHBOARD_URL}");
    }
}

fn open_browser(url: &str) {
    #[cfg(target_os = "macos")]
    {
        let _ = std::process::Command::new("open").arg(url).spawn();
    }
    #[cfg(target_os = "windows")]
    {
        let _ = std::process::Command::new("cmd").args(["/C", "start", "", url]).spawn();
    }
    #[cfg(all(unix, not(target_os = "macos")))]
    {
        let _ = std::process::Command::new("xdg-open").arg(url).spawn();
    }
}
