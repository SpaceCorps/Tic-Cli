//! Handler for generic `tic search <COLLECTION> <QUERY>` command.

use crate::cli::SearchArgs;
use crate::client::{Client, extract_hits};
use crate::commands::print;
use crate::error::Result;

pub fn run(args: SearchArgs, client: &Client) -> Result<()> {
    let res = client.search(
        &args.collection,
        &args.query,
        &args.query_by,
        args.filter_by.as_deref(),
        args.sort_by.as_deref(),
        Some(args.per_page),
    )?;
    print(extract_hits(&res));
    Ok(())
}
