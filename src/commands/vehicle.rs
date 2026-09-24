//! Handlers for `tic vehicle ...` commands.

use crate::cli::VehicleCommand;
use crate::client::{Client, extract_hits};
use crate::commands::print;
use crate::error::Result;

pub fn run(cmd: VehicleCommand, client: &Client) -> Result<()> {
    match cmd {
        VehicleCommand::Search { query, query_by, filter_by, sort_by, per_page } => {
            let res = client.search(
                "vehicles/se",
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
