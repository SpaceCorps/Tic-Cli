//! Handlers for `tic person ...` commands.

use crate::cli::PersonCommand;
use crate::client::{Client, seg};
use crate::commands::print;
use crate::error::Result;

pub fn run(cmd: PersonCommand, client: &Client) -> Result<()> {
    match cmd {
        PersonCommand::Search { query, page_size } => {
            let res = client.get(&format!("datasets/persons?query={}&pageSize={page_size}", seg(&query)))?;
            print(res);
        }
        PersonCommand::Get { person_id } => {
            let res = client.get(&format!("datasets/persons/{person_id}"))?;
            print(res);
        }
        PersonCommand::Companies { person_id, page_size } => {
            let res = client.get(&format!("datasets/persons/{person_id}/companies?pageSize={page_size}"))?;
            print(res);
        }
    }
    Ok(())
}
