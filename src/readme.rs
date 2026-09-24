//! The manual an agent reads before its first call. Markdown by default so it can be pasted
//! into a system prompt; `--json` gives the same rules as structured data.

use crate::{obj, output};

pub fn print() {
    if output::json() {
        output::write(&obj! {
            "tool" => "tic",
            "apiVersion" => API_VERSION,
            "rules" => RULES,
            "exitCodes" => obj! {
                "0" => "ok",
                "1" => "error - unclassified, report and stop",
                "2" => "network - retry once, then stop",
                "3" => "auth_required - stop, surface the remediation to a human",
                "4" => "not_found - do not retry",
                "5" => "rate_limited - back off before retrying",
                "6" => "invalid_input - fix the call",
                "7" => "no_account - run tic accounts list or tic login",
            },
        });
        return;
    }
    println!("{README}");
}

pub const API_VERSION: &str = "1.0.0";

const RULES: &[&str] = &[
    "Pass --account <name>, --api-key <key>, or set TIC_API_KEY environment variable.",
    "Run 'tic accounts list' to inspect configured accounts; ask the human which to use if uncertain.",
    "On code auth_required, stop and surface the remediation string. Do not retry.",
    "Company registration numbers can be formatted with or without hyphens (e.g. 556792-6687 or 5567926687).",
    "Use --json when parsing tool output programmatically or piping into jq.",
    "Bankruptcy and vehicle searches support full query-by, filter-by, and sort-by parameters.",
    "Person search supports personal identity numbers with '%' wildcards.",
];

const README: &str = r#"# tic - agent operating manual

A native Rust CLI for the TIC (tic.io) search and datasets API: Swedish company intelligence,
beneficial ownership, corporate graphs, debtor registries, bankruptcy filings, vehicle ownership,
and person lookups. Results are YAML on stdout, errors are YAML on stderr, and `--json` switches
both to structured JSON.

## Authentication & Accounts

Configure credentials via:
1. Keystore: `tic login <name>` or `tic accounts add <name> --api-key <key>`
2. Flag: `--api-key <key>` or `--account <name>` (short `-a`)
3. Environment: `export TIC_API_KEY="your-api-key"`

```bash
tic accounts list                     # what is configured here
tic login work                        # opens browser to copy API token
tic accounts add prod --api-key $KEY  # store key in OS keystore
```

## Company Commands

```bash
# Look up company by registration number
tic company get 556792-6687

# Look up company by internal TIC ID
tic company get-by-id 3325421

# Search companies by name or specific fields
tic company search "Bosma Interactive"
tic company search "+46767742725" --query-by "phoneNumbers.e164PhoneNumber"
tic company search "niels@example.com" --query-by "emailAddresses.emailAddress"
tic company search "Fabriksgatan" --query-by "mostRecentRegisteredAddress.streetAddress"
tic company search "*" --query-by "registrationNumber" --filter-by "hasIntelligence:true" --per-page 5

# Detailed lookups by company ID
tic company parties 3325421
tic company beneficial-owners 3325421
tic company intelligence 3325421
tic company graph 3325421
tic company tree 3325421
tic company debtor-summary 3325421
tic company vehicles 3325421
tic company properties 3325421
```

## Person Commands

```bash
# Search by personal identity number (use % for wildcard)
tic person search 198207174171

# Get person details
tic person get 1625054

# Get all companies where a person has a role
tic person companies 1625054
```

## Vehicle Commands

```bash
# Search by licence plate, VIN, manufacturer
tic vehicle search "ABC123"
tic vehicle search "YV1XZ" --query-by vin
```

## Bankruptcy Commands

```bash
# Search bankruptcy records
tic bankruptcy search "5566778899"
tic bankruptcy search "*" --filter-by "initiatedDate:>=1711929600" --sort-by "initiatedDate:desc"
```

## Generic Search

```bash
# Query any TIC collection directly
tic search companies "Volvo" --query-by "names.nameOrIdentifier"
```

## Output & Exit Codes

Add `--json` for raw structured JSON output:
```bash
tic company get 556792-6687 --json
```

| Exit Code | Name | Meaning |
|:---|:---|:---|
| 0 | `ok` | Command succeeded |
| 1 | `error` | General or unclassified error |
| 2 | `network` | Network or timeout error; retry once |
| 3 | `auth_required` | Missing or invalid API key |
| 4 | `not_found` | Entity not found; do not retry |
| 5 | `rate_limited` | Rate limit hit (429); back off |
| 6 | `invalid_input` | Parameter or validation error |
| 7 | `no_account` | No account specified |
"#;
