//! Handlers for `tic bankruptcy ...` commands.

use crate::cli::BankruptcyCommand;
use crate::client::{Client, extract_hits};
use crate::commands::print;
use crate::error::Result;

pub fn run(cmd: BankruptcyCommand, client: &Client) -> Result<()> {
    match cmd {
        BankruptcyCommand::Search { query, query_by, filter_by, sort_by, per_page } => {
            let res = client.search(
                "companies/bankruptcies/se",
                &query,
                &query_by,
                filter_by.as_deref(),
                sort_by.as_deref(),
                Some(per_page),
            )?;
            print(extract_hits(&res));
        }
    }
    Ok(())
}
