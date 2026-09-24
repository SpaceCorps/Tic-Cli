//! Multi-account management and API key resolution.
//!
//! Resolution priority:
//! 1. `--api-key <key>` explicit flag
//! 2. `--account <name>` explicit account name from keystore/config
//! 3. `TIC_API_KEY` environment variable
//! 4. If none provided, raises [`ErrorCode::NoAccount`] with available accounts.

use crate::client::Client;
use crate::config::{self, AccountConfig, Config};
use crate::error::{Error, ErrorCode, Result};
use crate::secrets;

pub struct Resolved {
    pub name: String,
    pub config: AccountConfig,
    pub api_key: String,
}

impl Resolved {
    pub fn client(&self) -> Client {
        Client::new(&self.api_key)
    }
}

pub fn resolve(account: Option<&str>, api_key: Option<&str>) -> Result<Resolved> {
    // 1. Explicit API key flag overrides everything
    if let Some(key) = api_key.map(str::trim).filter(|k| !k.is_empty()) {
        return Ok(Resolved { name: "api-key".into(), config: AccountConfig::default(), api_key: key.to_string() });
    }

    let config = config::load()?;

    // 2. Explicit account name
    if let Some(requested) = account.map(str::trim).filter(|s| !s.is_empty()) {
        let Some((name, acct)) = config.find(requested) else {
            return Err(Error::new(ErrorCode::NoAccount, format!("No account named '{requested}'."))
                .detail(describe(&config))
                .fix("tic accounts list"));
        };

        let key = secrets::store()?.get(&secrets::account_key(name))?;
        let Some(stored_key) = key.filter(|k| !k.trim().is_empty()) else {
            return Err(Error::new(ErrorCode::AuthRequired, format!("Account '{name}' has no stored API key."))
                .detail("The config entry exists but the keystore has nothing under it.")
                .fix(format!("tic accounts add {name} --api-key <key>")));
        };

        return Ok(Resolved { name: name.clone(), config: acct.clone(), api_key: stored_key });
    }

    // 3. TIC_API_KEY environment variable
    if let Ok(env_key) = std::env::var("TIC_API_KEY") {
        let trimmed = env_key.trim();
        if !trimmed.is_empty() {
            return Ok(Resolved { name: "env".into(), config: AccountConfig::default(), api_key: trimmed.to_string() });
        }
    }

    // 4. Missing account
    Err(Error::new(
        ErrorCode::NoAccount,
        "No account specified. Pass --account <name>, --api-key <key>, or set TIC_API_KEY.",
    )
    .detail(describe(&config))
    .fix("tic accounts list"))
}

fn describe(config: &Config) -> String {
    if config.accounts.is_empty() {
        return "No accounts are configured yet. Run 'tic accounts add <name> --api-key <key>' or 'tic login <name>'."
            .into();
    }
    let listed: Vec<String> = config
        .sorted()
        .into_iter()
        .map(|(k, v)| if v.identity.trim().is_empty() { k.clone() } else { format!("{k} ({})", v.identity) })
        .collect();
    format!("Configured accounts: {}", listed.join(", "))
}
