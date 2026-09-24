//! Subcommand implementations and dispatch.

pub mod accounts;
pub mod bankruptcy;
pub mod company;
pub mod login;
pub mod person;
pub mod search;
pub mod vehicle;

use serde_json::Value;

use crate::account;
use crate::cli::Command;
use crate::error::Result;

pub fn print(v: Value) {
    crate::output::write(&v);
}

pub fn run(command: Command, global_account: Option<String>, global_api_key: Option<String>) -> Result<()> {
    match command {
        Command::Company(cmd) => {
            let acct = account::resolve(global_account.as_deref(), global_api_key.as_deref())?;
            company::run(cmd, &acct.client())
        }
        Command::Bankruptcy(cmd) => {
            let acct = account::resolve(global_account.as_deref(), global_api_key.as_deref())?;
            bankruptcy::run(cmd, &acct.client())
        }
        Command::Person(cmd) => {
            let acct = account::resolve(global_account.as_deref(), global_api_key.as_deref())?;
            person::run(cmd, &acct.client())
        }
        Command::Vehicle(cmd) => {
            let acct = account::resolve(global_account.as_deref(), global_api_key.as_deref())?;
            vehicle::run(cmd, &acct.client())
        }
        Command::Search(args) => {
            let acct = account::resolve(global_account.as_deref(), global_api_key.as_deref())?;
            search::run(args, &acct.client())
        }
        Command::Login(args) => login::run(args),
        Command::Accounts(cmd) => accounts::run(cmd),
        Command::AgentReadme => {
            crate::readme::print();
            Ok(())
        }
    }
}
