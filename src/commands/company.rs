//! Handlers for `tic company ...` commands.

use crate::cli::CompanyCommand;
use crate::client::{Client, extract_hits, extract_single_hit};
use crate::commands::print;
use crate::error::Result;

pub fn run(cmd: CompanyCommand, client: &Client) -> Result<()> {
    match cmd {
        CompanyCommand::Get { reg_nr } => {
            let normalized = reg_nr.replace('-', "");
            let res = client.search("companies", &normalized, "registrationNumber", None, None, Some(1))?;
            let doc = extract_single_hit(&res)?;
            print(doc);
        }
        CompanyCommand::GetById { company_id } => {
            let filter = format!("companyId:[{company_id}]");
            let res = client.search("companies", "*", "registrationNumber", Some(&filter), None, Some(1))?;
            let doc = extract_single_hit(&res)?;
            print(doc);
        }
        CompanyCommand::Search { query, query_by, filter_by, sort_by, per_page } => {
            let res = client.search(
                "companies",
                &query,
                &query_by,
                filter_by.as_deref(),
                sort_by.as_deref(),
                Some(per_page),
            )?;
            print(extract_hits(&res));
        }
        CompanyCommand::Graph { company_id, include_beneficial_owners, max_nodes, max_roles } => {
            let mut path = format!(
                "datasets/companies/{company_id}/graph?includeBeneficialOwner={include_beneficial_owners}&maxTotalNodes={max_nodes}"
            );
            if let Some(roles) = max_roles {
                path.push_str(&format!("&limitMaxRoleInCompanies={roles}"));
            }
            let res = client.get(&path)?;
            print(res);
        }
        CompanyCommand::Tree { company_id } => {
            let res = client.get(&format!("datasets/companies/{company_id}/tree"))?;
            print(res);
        }
        CompanyCommand::Intelligence { company_id } => {
            let res = client.get(&format!("datasets/companies/{company_id}/intelligence"))?;
            print(res);
        }
        CompanyCommand::BeneficialOwners { company_id } => {
            let res = client.get(&format!("datasets/companies/{company_id}/se/beneficial-owners"))?;
            print(res);
        }
        CompanyCommand::Parties { company_id } => {
            let res = client.get(&format!("datasets/companies/{company_id}/parties"))?;
            print(res);
        }
        CompanyCommand::DebtorSummary { company_id } => {
            let res = client.get(&format!("datasets/companies/{company_id}/se/debtor-summary"))?;
            print(res);
        }
        CompanyCommand::Vehicles { company_id, page, page_size } => {
            let res = client
                .get(&format!("datasets/companies/{company_id}/se/vehicles?pageNumber={page}&pageSize={page_size}"))?;
            print(res);
        }
        CompanyCommand::Properties { company_id, include_historical } => {
            let current_only = !include_historical;
            let res = client.get(&format!(
                "datasets/companies/{company_id}/se/properties?includeOnlyCurrentOwnership={current_only}"
            ))?;
            print(res);
        }
    }
    Ok(())
}
